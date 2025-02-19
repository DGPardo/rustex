use std::{
    future::Future,
    str::FromStr,
    sync::{Arc, LazyLock},
};

use futures::StreamExt;
use rustex_core::prelude::*;
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
    async fn insert_order(user: UserId, client_order: ClientOrder) -> Result<OrderId, RustexError>;

    async fn get_user_orders(user: UserId) -> Result<Vec<OrderId>, RustexError>;

    async fn get_order_progress(
        user: UserId,
        order_id: OrderId,
        market: ExchangeMarket,
    ) -> Result<(bool, f64), RustexError>; // (is_pending, quantity_left)

    async fn try_delete_order(
        user: UserId,
        order_id: OrderId,
        market: ExchangeMarket,
    ) -> Result<bool, RustexError>;
}

#[derive(Clone)]
pub struct MatchingServer {
    pub exchange: ExchangeMarket,
    pub order_book: Arc<OrderBook>,
    pub db_rpc_client: Arc<DbServiceClient>,
}

impl MatchService for MatchingServer {
    async fn insert_order(
        self,
        c: Context,
        user_id: UserId,
        client_order: ClientOrder,
    ) -> Result<OrderId, RustexError> {
        // Optimisitc Strategy. Executing Matching and DB logging in parallel
        let (trades_fut, db_order) = match client_order.order_type {
            OrderType::Buy => {
                let buy_order: BuyOrder = self.order_book.into_order(client_order, user_id)?;
                (
                    tokio::task::spawn_blocking(move || self.order_book.process_order(buy_order)),
                    buy_order.0,
                )
            }
            OrderType::Sell => {
                let sell_order: SellOrder = self.order_book.into_order(client_order, user_id)?;
                (
                    tokio::task::spawn_blocking(move || self.order_book.process_order(sell_order)),
                    sell_order.0,
                )
            }
        };

        let db_client = Arc::clone(&self.db_rpc_client);
        let db_record = tokio::spawn(async move { db_client.insert_order(c, db_order).await });

        // Await concurrently the spawned tasks
        let (db_record, trades) = tokio::join!(db_record, trades_fut);

        // DANGER DANGER. What if only one of the two fails?
        // TODO: Handle errors properly
        let (_db_record, (trades, completed_orders)) = (db_record???, trades?);

        let db_client = Arc::clone(&self.db_rpc_client);
        tokio::spawn(async move {
            let r = db_client
                .insert_trades(c, self.exchange, trades, completed_orders)
                .await;
            match &r {
                Ok(Ok(_)) => (),
                _ => log::error!(
                    "An error happened when recording the trades in the DB: {:?}",
                    r
                ),
            }
        });
        Ok(db_order.order_id)
    }

    async fn get_order_progress(
        self,
        ctx: Context,
        user: UserId,
        order_id: OrderId,
        market: ExchangeMarket,
    ) -> Result<(/*is pending=*/ bool, /*quantity left=*/ f64), RustexError> {
        let order = self.db_rpc_client.get_orders(ctx, vec![order_id], market);
        let trades = self.db_rpc_client.get_order_trades(ctx, order_id, market);

        let (order, trades) = tokio::join!(order, trades);
        let (orders, trades) = (order??, trades??);

        if orders.len() != 1 {
            return Err(RustexError::DbServiceError(
                "Expecting to receive a single order".into(),
            ));
        }
        let order = orders.first().unwrap();
        if order.user_id != user {
            return Err(RustexError::UserFacingError(
                "The order id you requested does not match your user_id".into(),
            ));
        }
        if order.exchange != market {
            return Err(RustexError::DbServiceError(
                "Order exchange market do not match".into(),
            ));
        }

        let mut remaining = order.quantity;
        trades.into_iter().for_each(|q| {
            remaining -= q.quantity;
        });

        Ok((self.order_book.is_order_pending(order_id), remaining))
    }

    async fn get_user_orders(
        self,
        ctx: Context,
        user: UserId,
    ) -> Result<Vec<OrderId>, RustexError> {
        let db_orders = self
            .db_rpc_client
            .get_user_orders(ctx, user, self.exchange)
            .await??;
        Ok(db_orders)
    }

    async fn try_delete_order(
        self,
        ctx: Context,
        user: UserId,
        order_id: OrderId,
        market: ExchangeMarket,
    ) -> Result<bool, RustexError> {
        let registered_user = self
            .db_rpc_client
            .get_order_user(ctx, order_id, market)
            .await??; // O(1) in db
        if registered_user.is_some_and(|reg_user| reg_user == user) {
            if self.order_book.try_delete_order(order_id) {
                self.db_rpc_client
                    .insert_cancellation(ctx, market, order_id)
                    .await??;
                Ok(true)
            } else {
                Ok(false)
            }
        } else if registered_user.is_some() {
            Err(RustexError::AuthorizationError(
                "You are not authorized to cancel this order".into(),
            ))
        } else {
            Err(RustexError::UserFacingError(
                "Requested order is not associated with any user".into(),
            ))
        }
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
    let exchange = std::env::var("EXCHANGE_MARKET")
        .map(|env_var| ExchangeMarket::from_str(&env_var).unwrap())
        .expect("EXCHANGE_MARKET environment variable is not defined");
    let book = initialize_order_book(Arc::clone(&db_rpc_client), exchange).await;

    // TODO: Gather order book from database
    // TODO: Specify which order book (by currency, etc...)
    let state = MatchingServer {
        exchange,
        db_rpc_client,
        order_book: Arc::new(book),
    };

    let listener = create_tarpc_server!(ADDRESS.clone(), *MAX_NUMBER_CO_CONNECTIONS, state.clone());
    log::info!("Orders RPC:: listening on: {:?}", ADDRESS);
    listener.await
}

async fn initialize_order_book(
    db_rpc_client: Arc<DbServiceClient>,
    market: ExchangeMarket,
) -> OrderBook {
    let (last_order, last_trade, pending_orders_ids) = tokio::join!(
        db_rpc_client.get_last_order_id(Context::current()),
        db_rpc_client.get_last_trade_id(Context::current()),
        db_rpc_client.get_pending_orders_ids(Context::current(), market),
    );

    // Panic on startup if any of these cannot be retrieved
    let last_order = last_order
        .unwrap() // Fail startup
        .unwrap() // Fail startup
        .map(|e| e + 1)
        .unwrap_or(OrderId::from(0));
    let last_trade = last_trade
        .unwrap() // Fail startup
        .unwrap() // Fail startup
        .map(|e| e + 1)
        .unwrap_or(TradeId::from(0));

    let pending_order_ids = pending_orders_ids
        .expect("TARPC Error collecting pending order ids")
        .expect("Error Extracting pending order ids from the database");
    let pending_orders = db_rpc_client
        .get_orders(Context::current(), pending_order_ids, market)
        .await
        .expect("TARPC Failed to collect pending orders")
        .expect("DB Failed to collect pending orders");

    let (buy_orders, sell_orders): (Vec<Order>, Vec<Order>) = pending_orders
        .into_iter()
        .partition(|order| order.order_type == OrderType::Buy);
    let buy_orders: Vec<BuyOrder> = buy_orders.into_iter().map(BuyOrder::from).collect();
    let sell_orders: Vec<SellOrder> = sell_orders.into_iter().map(SellOrder::from).collect();

    macro_rules! update_book_orders {
        ($orders:ident, $fname:ident) => {{
            let mut order_tasks = JoinSet::new();
            for mut order in $orders {
                let db_client = Arc::clone(&db_rpc_client);
                order_tasks.spawn(async move {
                    let trades = db_client
                        .$fname(Context::current(), order.order_id, market)
                        .await??;
                    trades.into_iter().for_each(|trade| {
                        // Have to update the quantity remaining to be traded
                        order.quantity -= trade.quantity;
                    });
                    Ok::<_, RustexError>(order)
                });
            }
            order_tasks
        }};
    }

    let buy_orders = update_book_orders!(buy_orders, get_order_trades);
    let sell_orders = update_book_orders!(sell_orders, get_order_trades);
    let (buy_orders, sell_orders) = tokio::join!(buy_orders.join_all(), sell_orders.join_all());

    let buy_orders = buy_orders
        .into_iter()
        .map(|buy_order| buy_order.expect("Failed to sync a specific buy order")) // Panic if cannot be synced
        .collect::<Vec<_>>();
    let sell_orders = sell_orders
        .into_iter()
        .map(|sell_order| sell_order.expect("Failed to sync a specific sell order")) // Panic if cannot be synced
        .collect::<Vec<_>>();

    OrderBook::from_db(last_order, last_trade, buy_orders, sell_orders, market)
}
