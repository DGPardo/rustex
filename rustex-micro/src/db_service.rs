use diesel::{
    dsl::max, query_dsl::methods::SelectDsl, ExpressionMethods, JoinOnDsl, QueryDsl,
    SelectableHelper,
};
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection, RunQueryDsl,
};
use futures::StreamExt;
use rustex_core::{
    db::{
        self,
        models::{DbOrder, DbPendingOrder, DbTrade, OrderType},
    },
    prelude::{BuyOrder, ExchangeMarkets, Order, OrderId, SellOrder, Trade, TradeId},
};
use rustex_errors::RustexError;
use std::{future::Future, sync::LazyLock};
use tarpc::context::Context;

use crate::{create_tarpc_server, DEFAULT_ADDRESS, DEFAULT_MAX_NUMBER_CO_CONNECTIONS};

const DEFAULT_PORT: u16 = 6666;

pub static POSTGRES_ADDRESS: LazyLock<String> = LazyLock::new(|| {
    let address = std::env::var("POSTGRES_ADDRESS")
        .expect("POSTGRES_ADDRESS is not defined as an environment variable");
    let uname =
        std::env::var("PG_USERNAME").expect("PG_USERNAME not defined as environment variable");
    let passwd =
        std::env::var("PG_PASSWORD").expect("PG_PASSWORD not defined as environment variable");
    format!("postgres://{}:{}@{}", uname, passwd, address)
});

pub static ADDRESS: LazyLock<String> = LazyLock::new(|| {
    let addr = std::env::var("DATABASE_RPC_ADDRESS")
        .map(|addr| addr.into_boxed_str())
        .unwrap_or_else(|_| DEFAULT_ADDRESS.into());
    let port = std::env::var("DATABASE_RPC_PORT")
        .map(|port| port.parse().unwrap())
        .unwrap_or_else(|_| DEFAULT_PORT);
    format!("{}:{}", addr, port)
});

static MAX_NUMBER_CO_CONNECTIONS: LazyLock<usize> = LazyLock::new(|| {
    std::env::var("DB_RPC_MAX_NUMBER_CO_CONNECTIONS")
        .map(|n| n.parse().unwrap())
        .unwrap_or(DEFAULT_MAX_NUMBER_CO_CONNECTIONS)
});

#[tarpc::service]
pub trait DbService {
    /// Returns the last order id from the DB. It will be None if the table is emtpy
    async fn get_last_order_id() -> Result<Option<OrderId>, RustexError>;

    /// Returns the last trade id from the DB. It will be None if the table emtpy
    async fn get_last_trade_id() -> Result<Option<TradeId>, RustexError>;

    /// Returns all `buy` pending order ids
    async fn get_pending_buy_orders_ids() -> Result<Vec<OrderId>, RustexError>;

    /// Returns all `sell` pending order ids
    async fn get_pending_sell_orders_ids() -> Result<Vec<OrderId>, RustexError>;

    /// Returns the orders as requested by the user
    async fn get_orders(orders: Vec<OrderId>) -> Result<Vec<Order>, RustexError>;

    /// Returns the trades associated with the given `buy` order
    async fn get_buy_order_trades(order: OrderId) -> Result<Vec<Trade>, RustexError>;

    /// Returns the trades associated with the given `sell` order
    async fn get_sell_order_trades(order: OrderId) -> Result<Vec<Trade>, RustexError>;

    /// Records in the database a buying order
    async fn record_buy_order(
        exchange: ExchangeMarkets,
        buy_order: BuyOrder,
    ) -> Result<(), RustexError>;

    /// Records in the database a selling order
    async fn record_sell_order(
        exchange: ExchangeMarkets,
        sell_order: SellOrder,
    ) -> Result<(), RustexError>;

    /// Records in the database a Vector of trades
    async fn record_trades(
        exchange: ExchangeMarkets,
        trades: Vec<Trade>,
        completed_orders: Vec<OrderId>,
    ) -> Result<(), RustexError>;
}

#[derive(Clone)]
pub struct DbServer {
    pool: Pool<AsyncPgConnection>, // Clone only increases reference counting
}

impl DbServer {
    pub async fn new() -> Result<Self, RustexError> {
        let config =
            AsyncDieselConnectionManager::<AsyncPgConnection>::new(POSTGRES_ADDRESS.to_string());
        let pool = Pool::builder(config).build()?;
        Ok(Self { pool })
    }
}

macro_rules! insert_order {
    ($self:ident, $fname:ident, $order:ident) => {{
        let conn = &mut *$self.pool.get().await?;
        let order: DbOrder = DbOrder::from($order);

        let table_insert = diesel::insert_into(db::schema::orders::table)
            .values(&order)
            .returning(DbOrder::as_returning())
            .execute(conn);

        let order_id: DbPendingOrder = $order.id.into();
        let pending_insert = diesel::insert_into(db::schema::pending_orders::table)
            .values(&order_id)
            .returning(DbPendingOrder::as_returning())
            .execute(conn);

        let (inserted_rows, pending_ok) = tokio::join!(table_insert, pending_insert);
        let (inserted_rows, _pending_ok) = (inserted_rows?, pending_ok?);

        if inserted_rows != 1 {
            panic!("Failed to insert the order in the database");
        }
    }};
}

macro_rules! pending_orders {
    ($conn:ident, $order_type:ident) => {{
        use db::schema::*;
        let join = pending_orders::table
            .inner_join(orders::table.on(pending_orders::order_id.eq(orders::order_id)));
        let filter = QueryDsl::filter(join, orders::order_type.eq($order_type));
        let selection = QueryDsl::select(filter, orders::dsl::order_id);

        let p_orders: Vec<i64> = selection.load($conn).await?;
        Ok(p_orders.into_iter().map(|e| e.into()).collect())
    }};
}

macro_rules! pending_order_trades {
    ($conn:ident, $order_id:ident, $order_type:expr) => {{
        let order: i64 = $order_id.into();
        let rows: Vec<DbTrade> = db::schema::trades::dsl::trades
            .filter($order_type.eq(order))
            .load($conn)
            .await?;
        let rows: Vec<Trade> = rows.into_iter().map(Trade::from).collect();
        Ok(rows)
    }};
}

impl DbService for DbServer {
    async fn get_last_order_id(self, _: Context) -> Result<Option<OrderId>, RustexError> {
        let conn = &mut *self.pool.get().await?;
        use db::schema::orders::dsl::*;
        let max_order_id: Option<i64> =
            SelectDsl::select(orders, max(order_id)).first(conn).await?;
        Ok(max_order_id.map(|e| e.into()))
    }

    async fn get_last_trade_id(self, _: Context) -> Result<Option<TradeId>, RustexError> {
        let conn = &mut *self.pool.get().await?;
        use db::schema::trades::dsl::*;
        let max_trade_id: Option<i64> =
            SelectDsl::select(trades, max(trade_id)).first(conn).await?;
        Ok(max_trade_id.map(|e| e.into()))
    }

    async fn get_pending_buy_orders_ids(self, _: Context) -> Result<Vec<OrderId>, RustexError> {
        let conn = &mut *self.pool.get().await?;
        let order_type = OrderType::Buy;
        pending_orders!(conn, order_type)
    }

    async fn get_pending_sell_orders_ids(self, _: Context) -> Result<Vec<OrderId>, RustexError> {
        let conn = &mut *self.pool.get().await?;
        let order_type = OrderType::Sell;
        pending_orders!(conn, order_type)
    }

    async fn get_orders(
        self,
        _: Context,
        order_ids: Vec<OrderId>,
    ) -> Result<Vec<Order>, RustexError> {
        let conn = &mut *self.pool.get().await?;
        let order_ids = order_ids.into_iter().map(i64::from).collect::<Vec<_>>();

        use db::schema::orders::dsl::*;
        let rows: Vec<DbOrder> = orders.filter(order_id.eq_any(order_ids)).load(conn).await?;
        let rows: Vec<Order> = rows.into_iter().map(Order::from).collect();
        Ok(rows)
    }

    async fn get_buy_order_trades(
        self,
        _: Context,
        order: OrderId,
    ) -> Result<Vec<Trade>, RustexError> {
        let conn = &mut *self.pool.get().await?;
        use db::schema::trades::dsl::*;
        pending_order_trades!(conn, order, buy_order)
    }

    async fn get_sell_order_trades(
        self,
        _: Context,
        order: OrderId,
    ) -> Result<Vec<Trade>, RustexError> {
        let conn = &mut *self.pool.get().await?;
        use db::schema::trades::dsl::*;
        pending_order_trades!(conn, order, sell_order)
    }

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
        completed_orders: Vec<OrderId>,
    ) -> Result<(), RustexError> {
        let mut trades_conn = self.pool.get().await?;
        let mut completed_conn = self.pool.get().await?;

        let trades = tokio::spawn(async move {
            let trades = trades.into_iter().map(DbTrade::from).collect::<Vec<_>>();
            let inserted_rows = diesel::insert_into(db::schema::trades::table)
                .values(&trades)
                .returning(DbTrade::as_returning())
                .execute(&mut *trades_conn)
                .await?;
            if inserted_rows != trades.len() {
                panic!("Failed to insert the order in the database");
            }
            Ok::<(), RustexError>(())
        });
        let completed = tokio::spawn(async move {
            let completed_orders = completed_orders
                .into_iter()
                .map(i64::from)
                .collect::<Vec<_>>();
            let removed_rows = diesel::delete(
                db::schema::pending_orders::table
                    .filter(db::schema::pending_orders::order_id.eq_any(&completed_orders)),
            )
            .execute(&mut *completed_conn)
            .await?;
            if removed_rows != completed_orders.len() {
                panic!("Failed to delete completed orders from the pending orders table");
            }
            Ok::<(), RustexError>(())
        });

        let (trades, completed) = tokio::join!(trades, completed);
        let (_trades, _completed) = (trades??, completed??);
        Ok(())
    }
}

pub async fn start_service() {
    let state = DbServer::new()
        .await
        .expect("Failed to create database RPC server state");

    log::info!(
        "Testing connection with the database: {:?}",
        *POSTGRES_ADDRESS
    );
    let state = test_connection_to_db(state).await;
    log::info!("Database connection successful");

    let listener = create_tarpc_server!(ADDRESS.clone(), *MAX_NUMBER_CO_CONNECTIONS, state.clone());
    log::info!(
        "DB Service:: RPC listening on: {:?}. Database Address: {:?}",
        ADDRESS,
        *POSTGRES_ADDRESS
    );
    listener.await
}

async fn test_connection_to_db(db_server: DbServer) -> DbServer {
    let conn = &mut *db_server
        .pool
        .get()
        .await
        .expect("Failed to get pool client");
    let _result = diesel::sql_query("SELECT 1")
        .execute(conn)
        .await
        .expect("Failed to execute test query");
    db_server
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
            db_utc_tstamp_millis: None,
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
