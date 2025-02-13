use std::{
    future::Future,
    str::FromStr,
    sync::{Arc, LazyLock},
};

use futures::StreamExt;
use rustex_core::prelude::{
    BuyOrder, ExchangeMarkets, OrderBook, OrderId, SellOrder, Trade, TradeId, UserId,
};
use rustex_errors::RustexError;
use tarpc::context::Context;
use tokio::task::JoinSet;

use crate::{
    create_tarpc_server, db_service::DbServiceClient, get_db_service_client, DB_RPC_ADDRESS,
};
use crate::{DEFAULT_ADDRESS, DEFAULT_MAX_NUMBER_CO_CONNECTIONS};
const DEFAULT_PORT: u16 = 5555;

pub static ADDRESS: LazyLock<String> = LazyLock::new(|| {
    let addr = std::env::var("MATCH_RPC_ADDRESS")
        .map(|addr| addr.into_boxed_str())
        .unwrap_or_else(|_| DEFAULT_ADDRESS.into());
    let port = std::env::var("MATCH_RPC_PORT")
        .map(|addr| addr.parse().unwrap())
        .unwrap_or_else(|_| DEFAULT_PORT);
    format!("{}:{}", addr, port)
});

static MAX_NUMBER_CO_CONNECTIONS: LazyLock<usize> = LazyLock::new(|| {
    std::env::var("MATCH_RPC_MAX_NUMBER_CO_CONNECTIONS")
        .map(|n| n.parse().unwrap())
        .unwrap_or(DEFAULT_MAX_NUMBER_CO_CONNECTIONS)
});

#[tarpc::service]
pub trait MatchService {
    async fn insert_buy_order(
        user: UserId,
        price: i64,
        quantity: f64,
    ) -> Result<(OrderId, Vec<Trade>), RustexError>;

    async fn insert_sell_order(
        user: UserId,
        price: i64,
        quantity: f64,
    ) -> Result<(OrderId, Vec<Trade>), RustexError>;

    async fn get_user_orders(user: UserId) -> Result<Vec<OrderId>, RustexError>;

    async fn get_order_progress(user: UserId, order_id: OrderId) -> (bool, f64); // (is_pending, quantity_left)
}

#[derive(Clone)]
pub struct MatchingServer {
    pub exchange: ExchangeMarkets,
    pub order_book: Arc<OrderBook>,
    pub db_rpc_client: Arc<DbServiceClient>,
}

impl MatchService for MatchingServer {
    async fn insert_buy_order(
        self,
        c: Context,
        user_id: UserId,
        price: i64,
        quantity: f64,
    ) -> Result<(OrderId, Vec<Trade>), RustexError> {
        // Optimisitc Strategy. Executing Matching and DB logging in parallel
        let buy_order: BuyOrder = self.order_book.into_order(user_id, price, quantity);
        let trades_fut =
            tokio::task::spawn_blocking(move || self.order_book.process_order(buy_order));

        let db_client: Arc<DbServiceClient> = Arc::clone(&self.db_rpc_client);
        let db_record = tokio::spawn(async move {
            db_client
                .record_buy_order(c, self.exchange, buy_order)
                .await
        });

        // Await concurrently the spawned tasks
        let (db_record, trades) = tokio::join!(db_record, trades_fut);

        // DANGER DANGER. What if only one of the two fails?
        // TODO: Handle errors properly
        let (_db_record, (trades, completed_orders)) = (db_record???, trades?);

        let rpc_trades = trades.clone();
        let db_client: Arc<DbServiceClient> = Arc::clone(&self.db_rpc_client);
        tokio::spawn(async move {
            let r = db_client
                .record_trades(c, self.exchange, rpc_trades.clone(), completed_orders)
                .await;
            match &r {
                Ok(Ok(_)) => (),
                _ => log::error!(
                    "An error happened when recording the trades in the DB: {:?}",
                    r
                ),
            }
        });
        Ok((buy_order.id, trades))
    }

    async fn insert_sell_order(
        self,
        c: Context,
        user_id: UserId,
        price: i64,
        quantity: f64,
    ) -> Result<(OrderId, Vec<Trade>), RustexError> {
        // Optimisitc Strategy. Executing Matching and DB logging in parallel
        let sell_order: SellOrder = self.order_book.into_order(user_id, price, quantity);
        let trades_fut =
            tokio::task::spawn_blocking(move || self.order_book.process_order(sell_order));

        let db_client: Arc<DbServiceClient> = Arc::clone(&self.db_rpc_client);
        let db_record = tokio::spawn(async move {
            db_client
                .record_sell_order(c, self.exchange, sell_order)
                .await
        });

        // Await concurrently the spawned tasks
        let (db_record, trades) = tokio::join!(db_record, trades_fut);

        // DANGER DANGER. What if only one of the two fails?
        // TODO: Handle errors properly
        let (_db_record, (trades, completed_orders)) = (db_record???, trades?);

        let rpc_trades = trades.clone();
        let db_client: Arc<DbServiceClient> = Arc::clone(&self.db_rpc_client);
        tokio::spawn(async move {
            let r = db_client
                .record_trades(c, self.exchange, rpc_trades.clone(), completed_orders)
                .await;
            match &r {
                Ok(Ok(_)) => (),
                _ => log::error!(
                    "An error happened when recording the trades in the DB: {:?}",
                    r
                ),
            }
        });
        Ok((sell_order.id, trades))
    }

    async fn get_order_progress(
        self,
        _: Context,
        _user: UserId,
        _order_id: OrderId,
    ) -> (/*is pending=*/ bool, /*quantity left=*/ f64) {
        // TODO: Slow Path -> Query Database and do not block book matching progress
        (false, 0.0)
    }

    async fn get_user_orders(
        self,
        ctx: Context,
        user: UserId,
    ) -> Result<Vec<OrderId>, RustexError> {
        let db_orders = self.db_rpc_client.get_user_orders(ctx, user).await??;
        Ok(db_orders)
    }
}

pub async fn start_service() {
    let db_rpc_client = get_db_service_client(DB_RPC_ADDRESS.clone())
        .await
        .unwrap_or_else(|e| {
            panic!(
                "Failed to connect to db micro-service on address {:?}. Error: {:?}",
                DB_RPC_ADDRESS, e
            )
        });
    let db_rpc_client = Arc::new(db_rpc_client);
    let book = initialize_order_book(Arc::clone(&db_rpc_client)).await;

    // TODO: Gather order book from database
    // TODO: Specify which order book (by currency, etc...)
    let state = MatchingServer {
        exchange: std::env::var("EXCHANGE_MARKET")
            .map(|env_var| ExchangeMarkets::from_str(&env_var).unwrap())
            .expect("EXCHANGE_MARKET environment variable is not defined"),
        order_book: Arc::new(book),
        db_rpc_client,
    };

    let listener = create_tarpc_server!(ADDRESS.clone(), *MAX_NUMBER_CO_CONNECTIONS, state.clone());
    log::info!("Orders RPC:: listening on: {:?}", ADDRESS);
    listener.await
}

async fn initialize_order_book(db_rpc_client: Arc<DbServiceClient>) -> OrderBook {
    let (last_order, last_trade, buy_order_ids, sell_orders) = tokio::join!(
        db_rpc_client.get_last_order_id(Context::current()),
        db_rpc_client.get_last_trade_id(Context::current()),
        db_rpc_client.get_pending_buy_orders_ids(Context::current()),
        db_rpc_client.get_pending_sell_orders_ids(Context::current())
    );

    // Panic on startup if any of these cannot be retrieved
    let last_order = last_order
        .unwrap() // Fail startup
        .unwrap() // Fail startup
        .map(|mut e| {
            e.fetch_increment();
            e
        })
        .unwrap_or(OrderId::from(0));
    let last_trade = last_trade
        .unwrap() // Fail startup
        .unwrap() // Fail startup
        .map(|mut e| {
            e.fetch_increment();
            e
        })
        .unwrap_or(TradeId::from(0));
    let buy_order_ids = buy_order_ids.unwrap().unwrap();
    let sell_order_ids = sell_orders.unwrap().unwrap();

    let (buy_orders, sell_orders) = tokio::join!(
        db_rpc_client.get_orders(Context::current(), buy_order_ids),
        db_rpc_client.get_orders(Context::current(), sell_order_ids)
    );

    let buy_orders: Vec<BuyOrder> = buy_orders
        .unwrap()
        .unwrap()
        .into_iter()
        .map(BuyOrder::from)
        .collect();

    let sell_orders: Vec<SellOrder> = sell_orders
        .unwrap()
        .unwrap()
        .into_iter()
        .map(SellOrder::from)
        .collect();

    macro_rules! update_book_orders {
        ($orders:ident, $fname:ident) => {{
            let mut order_tasks = JoinSet::new();
            for mut order in $orders {
                let db_client = Arc::clone(&db_rpc_client);
                order_tasks.spawn(async move {
                    let trades = db_client.$fname(Context::current(), order.id).await??;
                    trades.into_iter().for_each(|trade| {
                        order.quantity -= trade.quantity;
                    });
                    Ok::<_, RustexError>(order)
                });
            }
            order_tasks
        }};
    }

    let buy_orders = update_book_orders!(buy_orders, get_buy_order_trades);
    let sell_orders = update_book_orders!(sell_orders, get_sell_order_trades);
    let (buy_orders, sell_orders) = tokio::join!(buy_orders.join_all(), sell_orders.join_all());

    let buy_orders = buy_orders
        .into_iter()
        .map(|buy_order| buy_order.expect("Failed to sync a specific buy order")) // Panic if cannot be synced
        .collect::<Vec<_>>();
    let sell_orders = sell_orders
        .into_iter()
        .map(|sell_order| sell_order.expect("Failed to sync a specific sell order")) // Panic if cannot be synced
        .collect::<Vec<_>>();

    OrderBook::from_db(last_order, last_trade, buy_orders, sell_orders)
}
