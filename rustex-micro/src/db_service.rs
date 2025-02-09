use diesel::SelectableHelper;
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection, RunQueryDsl,
};
use futures::{future, StreamExt};
use rustex_core::{
    db::{
        self,
        models::{DbOrder, DbTrade},
    },
    prelude::{BuyOrder, ExchangeMarkets, SellOrder, Trade},
};
use rustex_errors::RustexError;
use std::{
    future::Future,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
    sync::LazyLock,
};
use tarpc::{context::Context, tokio_serde::formats::Json};

use crate::{DEFAULT_ADDRESS, DEFAULT_MAX_NUMBER_CO_CONNECTIONS};

const DEFAULT_PORT: u16 = 6666;

pub static DATABASE_ADDRESS: LazyLock<Box<str>> = LazyLock::new(|| {
    let addr = std::env::var("POSTGRES_ADDRESS")
        .expect("POSTGRES_ADDRESS is not defined as an environment variable");
    addr.into_boxed_str()
});

pub static ADDRESS: LazyLock<(IpAddr, u16)> = LazyLock::new(|| {
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
    async fn record_buy_order(
        exchange: ExchangeMarkets,
        buy_order: BuyOrder,
    ) -> Result<(), RustexError>;
    async fn record_sell_order(
        exchange: ExchangeMarkets,
        sell_order: SellOrder,
    ) -> Result<(), RustexError>;
    async fn record_trades(
        exchange: ExchangeMarkets,
        trades: Vec<Trade>,
    ) -> Result<(), RustexError>;
}

#[derive(Clone)]
pub struct DbServer {
    pool: Pool<AsyncPgConnection>, // Clone only increases reference counting
}

impl DbServer {
    pub async fn new() -> Result<Self, RustexError> {
        let config =
            AsyncDieselConnectionManager::<AsyncPgConnection>::new(DATABASE_ADDRESS.to_string());
        let pool = Pool::builder(config).build()?;
        Ok(Self { pool })
    }
}

macro_rules! insert_order {
    ($self:ident, $fname:ident, $order:ident) => {{
        let conn = &mut *$self.pool.get().await?;
        let order: DbOrder = DbOrder::from($order);
        let inserted_rows = diesel::insert_into(db::schema::orders::table)
            .values(&order)
            .returning(DbOrder::as_returning())
            .on_conflict_do_nothing()
            .execute(conn)
            .await?;

        if inserted_rows != 1 {
            panic!("Failed to insert the order in the database");
        }
    }};
}

impl DbService for DbServer {
    async fn record_buy_order(
        self,
        _: Context,
        _exchange: ExchangeMarkets,
        buy: BuyOrder,
    ) -> Result<(), RustexError> {
        insert_order!(self, from_buy, buy);
        Ok(())
    }

    async fn record_sell_order(
        self,
        _: Context,
        _exchange: ExchangeMarkets,
        sell: SellOrder,
    ) -> Result<(), RustexError> {
        insert_order!(self, from_sell, sell);
        Ok(())
    }

    async fn record_trades(
        self,
        _: Context,
        _exchange: ExchangeMarkets,
        trades: Vec<Trade>,
    ) -> Result<(), RustexError> {
        let conn = &mut *self.pool.get().await?;
        let trades = trades.into_iter().map(DbTrade::from).collect::<Vec<_>>();
        let inserted_rows = diesel::insert_into(db::schema::trades::table)
            .values(&trades)
            .returning(DbTrade::as_returning())
            .on_conflict_do_nothing()
            .execute(conn)
            .await?;
        if inserted_rows != trades.len() {
            panic!("Failed to insert the order in the database");
        }
        Ok(())
    }
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
    use super::*;
    use rustex_core::prelude::{Order, OrderId, UserId};
    use std::time::Duration;
    use tarpc::context;

    #[tokio::test]
    async fn test_insert_buy_order() {
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
        };

        let r = client
            .record_buy_order(
                context::current(),
                ExchangeMarkets::BTC_USD,
                BuyOrder::from(order),
            )
            .await;

        assert!(r.is_ok());
        assert!(r.unwrap().is_ok());

        assert!(!server.is_finished());
        server.abort();
    }
}
