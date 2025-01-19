use futures::{future, StreamExt};
use rustex_core::prelude::{EpochTime, OrderBook, OrderId, Trade, UserId};
use std::{
    future::Future,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
    sync::{Arc, LazyLock},
};
use tarpc::{
    context::Context,
    server::{self, Channel},
    tokio_serde::formats::Json,
};

const DEFAULT_ADDRESS: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 5555;
const DEFAULT_MAX_NUMBER_CO_CONNECTIONS: usize = 10_000;

static ADDRESS: LazyLock<(IpAddr, u16)> = LazyLock::new(|| {
    let addr = std::env::var("MATCH_RPC_ADDRESS")
        .map(|addr| addr.into_boxed_str())
        .unwrap_or_else(|_| DEFAULT_ADDRESS.into());
    let port = std::env::var("MATCH_RPC_PORT")
        .map(|addr| addr.parse().unwrap())
        .unwrap_or_else(|_| DEFAULT_PORT);
    (IpAddr::from(Ipv4Addr::from_str(&addr).unwrap()), port)
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
        price: u64,
        quantity: f64,
        time: EpochTime,
    ) -> (OrderId, Vec<Trade>);
    async fn insert_sell_order(
        user: UserId,
        price: u64,
        quantity: f64,
        time: EpochTime,
    ) -> (OrderId, Vec<Trade>);
}

#[derive(Clone)]
pub struct MatchingServer(Arc<OrderBook>);

impl MatchService for MatchingServer {
    async fn insert_buy_order(
        self,
        _: Context,
        user_id: UserId,
        price: u64,
        quantity: f64,
        time: EpochTime,
    ) -> (OrderId, Vec<Trade>) {
        self.0.insert_buy_order(user_id, price, quantity, time)
    }

    async fn insert_sell_order(
        self,
        _: Context,
        user_id: UserId,
        price: u64,
        quantity: f64,
        time: EpochTime,
    ) -> (OrderId, Vec<Trade>) {
        self.0.insert_sell_order(user_id, price, quantity, time)
    }
}

#[tokio::main]
pub async fn main() {
    let mut listener = tarpc::serde_transport::tcp::listen(*ADDRESS, Json::default)
        .await
        .unwrap();

    log::info!("Orders RPC listening on: {:?}", ADDRESS);

    // TODO: Gather order book from database
    let state = MatchingServer(Arc::new(OrderBook::default()));

    listener.config_mut().max_frame_length(u32::MAX as usize);
    listener
        // Ignore accept errors.
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
