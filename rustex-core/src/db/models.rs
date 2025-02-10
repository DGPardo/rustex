use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

use crate::prelude::{BuyOrder, Order, OrderId, SellOrder, Trade};

#[derive(DbEnum, Debug, Serialize, Deserialize)]
#[ExistingTypePath = "crate::db::schema::sql_types::Ordertype"]
#[DbValueStyle = "snake_case"]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = super::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbOrder {
    pub order_id: i64,
    pub user_id: i64,
    pub price: i64,
    pub quantity: f64,
    pub created_at: Option<DateTime<Utc>>, // Diesel automatically handles time-zone conversions
    pub order_type: OrderType,
}

impl From<SellOrder> for DbOrder {
    fn from(order: SellOrder) -> Self {
        Self {
            order_id: order.id.into(),
            user_id: order.user_id.into(),
            price: order.price,
            quantity: order.quantity,
            order_type: OrderType::Sell,
            created_at: None,
        }
    }
}

impl From<BuyOrder> for DbOrder {
    fn from(order: BuyOrder) -> Self {
        Self {
            order_id: order.id.into(),
            user_id: order.user_id.into(),
            price: order.price,
            quantity: order.quantity,
            order_type: OrderType::Buy,
            created_at: None,
        }
    }
}

impl From<DbOrder> for Order {
    fn from(value: DbOrder) -> Self {
        Self {
            id: value.order_id.into(),
            user_id: value.user_id.into(),
            price: value.price,
            quantity: value.quantity,
            db_utc_tstamp_millis: value.created_at.map(|e| e.timestamp_millis()),
        }
    }
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = super::schema::trades)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbTrade {
    trade_id: i64,
    buy_order: i64,
    sell_order: i64,
    price: i64,
    quantity: f64,
    created_at: Option<DateTime<Utc>>, // Diesel automatically handles time-zone conversions
}

impl From<Trade> for DbTrade {
    fn from(t: Trade) -> Self {
        Self {
            trade_id: t.id.into(),
            buy_order: t.buy_order_id.into(),
            sell_order: t.sell_order_id.into(),
            price: t.price,
            quantity: t.quantity,
            created_at: None,
        }
    }
}

impl From<DbTrade> for Trade {
    fn from(t: DbTrade) -> Self {
        Self {
            id: t.trade_id.into(),
            buy_order_id: t.buy_order.into(),
            sell_order_id: t.sell_order.into(),
            price: t.price,
            quantity: t.quantity,
        }
    }
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = super::schema::pending_orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbPendingOrder {
    order_id: i64,
}

impl From<OrderId> for DbPendingOrder {
    fn from(value: OrderId) -> Self {
        DbPendingOrder {
            order_id: value.into(),
        }
    }
}
