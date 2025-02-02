use dotenvy::dotenv;
use futures::{future, StreamExt};
use rustex_core::prelude::EpochTime;
use rustex_errors::RustexError;
use std::{
    future::Future,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
    sync::LazyLock,
};
use tarpc::{
    context::Context,
    server::{self, Channel},
    tokio_serde::formats::Json,
};

const DEFAULT_ADDRESS: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 7777;
const DEFAULT_MAX_NUMBER_CO_CONNECTIONS: usize = 10_000;

static ADDRESS: LazyLock<(IpAddr, u16)> = LazyLock::new(|| {
    let addr = std::env::var("TIME_RPC_ADDRESS")
        .map(|addr| addr.into_boxed_str())
        .unwrap_or_else(|_| DEFAULT_ADDRESS.into());
    let port = std::env::var("TIME_RPC_PORT")
        .map(|addr| addr.parse().unwrap())
        .unwrap_or_else(|_| DEFAULT_PORT);
    (IpAddr::from(Ipv4Addr::from_str(&addr).unwrap()), port)
});

static MAX_NUMBER_CO_CONNECTIONS: LazyLock<usize> = LazyLock::new(|| {
    std::env::var("TIME_RPC_MAX_NUMBER_CO_CONNECTIONS")
        .map(|n| n.parse().unwrap())
        .unwrap_or(DEFAULT_MAX_NUMBER_CO_CONNECTIONS)
});

#[tarpc::service]
pub trait TimeService {
    /// Returns the order id
    async fn get_time() -> Result<EpochTime, RustexError>;
}

#[derive(Clone)]
pub struct TimeServer; // stateless

impl TimeService for TimeServer {
    async fn get_time(self, _: Context) -> Result<EpochTime, RustexError> {
        Ok(EpochTime::now()?)
    }
}

#[tokio::main]
pub async fn main() {
    dotenv().unwrap();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    start_service().await
}

pub async fn start_service() {
    let mut listener = tarpc::serde_transport::tcp::listen(*ADDRESS, Json::default)
        .await
        .unwrap();

    log::info!("Time Service:: RPC listening on: {:?}", ADDRESS);

    listener.config_mut().max_frame_length(u32::MAX as usize);
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        .map(|channel| {
            let server = TimeServer {};
            channel.execute(server.serve()).for_each(spawn)
        })
        .buffered(*MAX_NUMBER_CO_CONNECTIONS) // in order
        .for_each(|_| async {})
        .await;
}

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}
