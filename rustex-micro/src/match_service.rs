use crate::{db_service::DbServiceClient, get_db_service_client, DB_RPC_ADDRESS};
use futures::{future, StreamExt};
use rustex_core::prelude::{
    BuyOrder, ExchangeMarkets, OrderBook, OrderId, SellOrder, Trade, UserId,
};
use rustex_errors::RustexError;
use std::{
    future::Future,
    sync::{Arc, LazyLock},
};
use tarpc::{
    context::Context,
    server::{self, Channel},
    tokio_serde::formats::Json,
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
        let (_db_record, trades) = (db_record???, trades?);

        let rpc_trades = trades.clone();
        let db_client: Arc<DbServiceClient> = Arc::clone(&self.db_rpc_client);
        tokio::spawn(async move {
            let r = db_client
                .record_trades(c, self.exchange, rpc_trades.clone())
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
        let (_db_record, trades) = (db_record???, trades?);

        let rpc_trades = trades.clone();
        let db_client: Arc<DbServiceClient> = Arc::clone(&self.db_rpc_client);
        tokio::spawn(async move {
            let r = db_client
                .record_trades(c, self.exchange, rpc_trades.clone())
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
}

pub async fn start_service() {
    let mut listener = tarpc::serde_transport::tcp::listen(ADDRESS.clone(), Json::default)
        .await
        .unwrap();

    log::info!("Orders RPC:: listening on: {:?}", ADDRESS);

    let db_rpc_client = get_db_service_client(DB_RPC_ADDRESS.clone())
        .await
        .unwrap_or_else(|e| {
            panic!(
                "Failed to connect to db micro-service on address {:?}. Error: {:?}",
                DB_RPC_ADDRESS, e
            )
        });
    let db_rpc_client = Arc::new(db_rpc_client);

    // TODO: Gather order book from database
    // TODO: Specify which order book (by currency, etc...)
    let state = MatchingServer {
        exchange: ExchangeMarkets::BTC_USD, // TODO: Either Env Arg or CLI Arg
        order_book: Arc::new(OrderBook::default()),
        db_rpc_client,
    };

    listener.config_mut().max_frame_length(u32::MAX as usize);
    listener
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        .map(|channel| channel.execute(state.clone().serve()).for_each(spawn))
        .buffered(*MAX_NUMBER_CO_CONNECTIONS) // in order
        .for_each(|_| async {})
        .await;
}

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}
