use std::{future::Future, sync::LazyLock};

use diesel::{dsl::max, BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection, RunQueryDsl,
};
use futures::StreamExt;
use rustex_core::{db, prelude::*};
use rustex_errors::RustexError;
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

    /// Returns all pending order ids
    async fn get_pending_orders_ids(market: ExchangeMarket) -> Result<Vec<OrderId>, RustexError>;

    /// Return all the orders of a given user
    async fn get_user_orders(
        user: UserId,
        market: ExchangeMarket,
    ) -> Result<Vec<OrderId>, RustexError>;

    /// Return the user for a specific order
    async fn get_order_user(
        order_id: OrderId,
        market: ExchangeMarket,
    ) -> Result<Option<UserId>, RustexError>;

    /// Returns the orders requested by the user
    async fn get_orders(
        orders: Vec<OrderId>,
        market: ExchangeMarket,
    ) -> Result<Vec<Order>, RustexError>;

    /// Returns the trades associated with the given `buy` order
    async fn get_order_trades(
        order: OrderId,
        market: ExchangeMarket,
    ) -> Result<Vec<Trade>, RustexError>;

    /// Insert in the database a new order
    async fn insert_order(order: Order) -> Result<(), RustexError>;

    /// Inserts in the database a new list of trades.
    /// Also, removes from the pending orders table those that are completed
    async fn insert_trades(
        market: ExchangeMarket,
        trades: Vec<Trade>,
        completed_orders: Vec<OrderId>,
    ) -> Result<(), RustexError>;

    /// Insert a new cancellation
    async fn insert_cancellation(market: ExchangeMarket, order: OrderId)
        -> Result<(), RustexError>;
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

impl DbService for DbServer {
    async fn get_last_order_id(self, _: Context) -> Result<Option<OrderId>, RustexError> {
        let conn = &mut *self.pool.get().await?;
        use db::schema::orders::dsl::*;
        let max_order_id: Option<i64> = orders.select(max(order_id)).first(conn).await?;
        Ok(max_order_id.map(|e| e.into()))
    }

    async fn get_last_trade_id(self, _: Context) -> Result<Option<TradeId>, RustexError> {
        let conn = &mut *self.pool.get().await?;
        use db::schema::trades::dsl::*;
        let max_trade_id: Option<i64> = trades.select(max(trade_id)).first(conn).await?;
        Ok(max_trade_id.map(|e| e.into()))
    }

    async fn get_pending_orders_ids(
        self,
        _: Context,
        market: ExchangeMarket,
    ) -> Result<Vec<OrderId>, RustexError> {
        let conn = &mut *self.pool.get().await?;
        use db::schema::pending_orders::dsl::*;
        let query = diesel::QueryDsl::select(pending_orders.filter(exchange.eq(market)), order_id);
        let order_ids: Vec<i64> = query.load(conn).await?;
        Ok(order_ids.into_iter().map(|e| e.into()).collect())
    }

    async fn get_user_orders(
        self,
        _: Context,
        user: UserId,
        market: ExchangeMarket,
    ) -> Result<Vec<OrderId>, RustexError> {
        let conn = &mut *self.pool.get().await?;

        use db::schema::orders::dsl::*;
        let query = orders
            .filter(user_id.eq(user).and(exchange.eq(market)))
            .select(order_id);
        let order_ids: Vec<OrderId> = query.load(conn).await?;

        Ok(order_ids)
    }

    async fn get_order_user(
        self,
        _: Context,
        order_identifier: OrderId,
        market: ExchangeMarket,
    ) -> Result<Option<UserId>, RustexError> {
        let conn = &mut *self.pool.get().await?;

        use db::schema::orders::dsl::*;
        let query = orders
            .filter(order_id.eq(order_identifier).and(exchange.eq(market)))
            .select(user_id);
        let user_ids = query.load(conn).await?;

        match user_ids.len() {
            0 => Ok(None),
            1 => Ok(Some(user_ids[0])),
            _ => Err(RustexError::DbServiceError(
                "There are multiple users for a single order id".into(),
            )),
        }
    }

    async fn get_orders(
        self,
        _: Context,
        order_ids: Vec<OrderId>,
        market: ExchangeMarket,
    ) -> Result<Vec<Order>, RustexError> {
        let conn = &mut *self.pool.get().await?;
        let order_ids = order_ids.into_iter().map(i64::from).collect::<Vec<_>>();

        use db::schema::orders::dsl::*;
        let rows: Vec<Order> = orders
            .filter(exchange.eq(market).and(order_id.eq_any(order_ids)))
            .load(conn)
            .await?;
        Ok(rows)
    }

    async fn get_order_trades(
        self,
        _: Context,
        order: OrderId,
        market: ExchangeMarket,
    ) -> Result<Vec<Trade>, RustexError> {
        let conn = &mut *self.pool.get().await?;

        let order: i64 = order.into();

        use db::schema::trades::dsl::*;
        let query = trades.filter(
            exchange
                .eq(market)
                .and(buy_order.eq(order).or(sell_order.eq(order))),
        );
        let rows: Vec<Trade> = query.load(conn).await?;
        Ok(rows)
    }

    async fn insert_order(self, _: Context, new_order: Order) -> Result<(), RustexError> {
        let conn = &mut *self.pool.get().await?;

        // Insert new order
        let insert_new_order = {
            use db::schema::orders::dsl::*;
            diesel::insert_into(orders).values(&new_order).execute(conn)
        };

        // Insert new pending order
        let pending_order = PendingOrder {
            order_id: new_order.order_id,
            exchange: new_order.exchange,
        };
        let insert_new_pending_order = {
            use db::schema::pending_orders::dsl::*;
            diesel::insert_into(pending_orders)
                .values(&pending_order)
                .execute(conn)
        };

        let (insert_new_order, insert_new_pending_order) =
            tokio::join!(insert_new_order, insert_new_pending_order);

        // Raise errors
        insert_new_order?;
        insert_new_pending_order?;

        Ok(())
    }

    async fn insert_trades(
        self,
        _: Context,
        market: ExchangeMarket,
        trades: Vec<Trade>,
        completed_orders: Vec<OrderId>,
    ) -> Result<(), RustexError> {
        let mut conn = self.pool.get().await?;

        // Insertion in Trades Table
        let inserted_trades = {
            diesel::insert_into(db::schema::trades::table)
                .values(&trades)
                .returning(Trade::as_returning())
                .execute(&mut *conn)
        };

        // Removing from Pending Orders Table
        let marked_completed = {
            use db::schema::pending_orders::dsl::*;
            diesel::delete(
                pending_orders.filter(order_id.eq_any(&completed_orders).and(exchange.eq(market))),
            )
            .execute(&mut *conn)
        };

        let (inserted_trades, marked_completed) = tokio::join!(inserted_trades, marked_completed);
        let (inserted_trades, marked_completed) = (inserted_trades?, marked_completed?);

        // Final error checking
        if inserted_trades != trades.len() {
            return Err(RustexError::DbServiceError(
                "Failed to insert all orders in the database".into(),
            ));
        }

        if marked_completed != completed_orders.len() {
            return Err(RustexError::DbServiceError(
                "Failed to delete completed orders from the pending orders table".into(),
            ));
        }

        Ok(())
    }

    async fn insert_cancellation(
        self,
        _: Context,
        market: ExchangeMarket,
        order_id: OrderId,
    ) -> Result<(), RustexError> {
        let mut conn = self.pool.get().await?;

        let cancellation = CancelledOrder {
            order_id,
            exchange: market,
            created_at: None,
        };
        let insertion = diesel::insert_into(db::schema::cancelled_orders::table)
            .values(&cancellation)
            .returning(CancelledOrder::as_returning())
            .execute(&mut *conn)
            .await?;
        if insertion != 1 {
            return Err(RustexError::DbServiceError(
                "Failed to record cancelled order".into(),
            ));
        }

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
