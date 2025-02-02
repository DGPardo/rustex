use diesel::SelectableHelper;
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection, RunQueryDsl,
};
use dotenvy::dotenv;
use futures::{future, StreamExt};
use rustex_core::{
    db::models::DbOrder,
    prelude::{db, BuyOrder, SellOrder},
};
use rustex_errors::RustexError;
use std::{
    future::Future,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
    sync::LazyLock,
};
use tarpc::{context::Context, tokio_serde::formats::Json};
use uuid::Uuid;

const DEFAULT_ADDRESS: &str = "127.0.0.1"; // Of this microservice
const DEFAULT_PORT: u16 = 6666;
const DEFAULT_MAX_NUMBER_CO_CONNECTIONS: usize = 10_000;

static DATABASE_ADDRESS: LazyLock<Box<str>> = LazyLock::new(|| {
    let addr = std::env::var("POSTGRES_ADDRESS")
        .expect("POSTGRES_ADDRESS is not defined as an environment variable");
    addr.into_boxed_str()
});

static ADDRESS: LazyLock<(IpAddr, u16)> = LazyLock::new(|| {
    let addr = std::env::var("DATABASE_RPC_ADDRESS")
        .map(|addr| addr.into_boxed_str())
        .unwrap_or_else(|_| DEFAULT_ADDRESS.into());
    let port = std::env::var("DATABASE_RPC_PORT")
        .map(|addr| addr.parse().unwrap())
        .unwrap_or_else(|_| DEFAULT_PORT);
    (IpAddr::from(Ipv4Addr::from_str(&addr).unwrap()), port)
});

static MAX_NUMBER_CO_CONNECTIONS: LazyLock<usize> = LazyLock::new(|| {
    std::env::var("DB_RPC_MAX_NUMBER_CO_CONNECTIONS")
        .map(|n| n.parse().unwrap())
        .unwrap_or(DEFAULT_MAX_NUMBER_CO_CONNECTIONS)
});

#[tarpc::service]
pub trait DbService {
    async fn record_buy_order(buy_order: BuyOrder) -> Result<Uuid, RustexError>;
    async fn record_sell_order(sell_order: SellOrder) -> Result<Uuid, RustexError>;
}

#[derive(Clone)]
struct DbServer {
    pool: Pool<AsyncPgConnection>, // Clone only increases reference counting
}

impl DbServer {
    async fn new() -> Result<Self, RustexError> {
        let config =
            AsyncDieselConnectionManager::<AsyncPgConnection>::new(DATABASE_ADDRESS.to_string());
        let pool = Pool::builder(config).build()?;
        Ok(Self { pool })
    }
}

macro_rules! insert_order {
    ($self:ident, $fname:ident, $order:ident) => {{
        let conn = &mut *$self.pool.get().await.unwrap();

        let order_id = Uuid::new_v4(); // TODO: check with DB for clashes
        let mut order: DbOrder = $fname(order_id, $order);

        let mut inserted_rows = 0;
        while inserted_rows == 0 {
            inserted_rows += diesel::insert_into(db::schema::orders::table)
                .values(&order)
                .returning(DbOrder::as_returning())
                .on_conflict_do_nothing()
                .execute(conn)
                .await?;

            log::warn!("Order UUID Clash detected. Retrying...");
            order.order_id = Uuid::new_v4();
        }
        Ok(order_id)
    }};
}

impl DbService for DbServer {
    async fn record_buy_order(self, _: Context, buy: BuyOrder) -> Result<Uuid, RustexError> {
        let from_buy = db::models::DbOrder::from_buy_order;
        insert_order!(self, from_buy, buy)
    }

    async fn record_sell_order(self, _: Context, sell: SellOrder) -> Result<Uuid, RustexError> {
        let from_sell = db::models::DbOrder::from_sell_order;
        insert_order!(self, from_sell, sell)
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

    log::info!(
        "DB Service:: RPC listening on: {:?}. Database Address: {:?}",
        ADDRESS,
        *DATABASE_ADDRESS
    );

    let state = DbServer::new()
        .await
        .expect("Failed to create Database RPC Server State");

    listener.config_mut().max_frame_length(u32::MAX as usize);
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(tarpc::server::BaseChannel::with_defaults)
        .map(|channel| {
            let state = state.clone();
            tarpc::server::Channel::execute(channel, state.serve()).for_each(spawn)
        })
        .buffered(*MAX_NUMBER_CO_CONNECTIONS) // in order
        .for_each(|_| async {})
        .await;
}

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

#[cfg(test)]
mod test {
    use std::time::Duration;
    use tarpc::context;
    use rustex_core::prelude::{EpochTime, Order, OrderId, UserId};
    use super::*;

    #[tokio::test]
    async fn test_insert_buy_order() {
        dotenv().unwrap();
        let server = tokio::spawn(start_service());
        tokio::time::sleep(Duration::from_secs(1)).await;

        let mut transport = tarpc::serde_transport::tcp::connect(
            &*ADDRESS,
            tarpc::tokio_serde::formats::Json::default,
        );
        transport.config_mut().max_frame_length(u32::MAX as usize);

        let client =
            DbServiceClient::new(tarpc::client::Config::default(), transport.await.unwrap())
                .spawn();

        let order = Order {
            id: OrderId::from(0),
            user_id: UserId::from(1),
            price: 2,
            quantity: 3.0,
            unix_epoch: EpochTime::now().unwrap(),
        };

        let r = client
            .record_buy_order(context::current(), BuyOrder::from(order))
            .await;

        assert!(r.is_ok());
        assert!(r.unwrap().is_ok());

        assert!(!server.is_finished());
        server.abort();

    }
}
