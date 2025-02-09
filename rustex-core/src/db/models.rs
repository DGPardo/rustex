use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;

use crate::prelude::{BuyOrder, SellOrder, Trade};

#[derive(DbEnum, Debug)]
#[ExistingTypePath = "crate::db::schema::sql_types::Ordertype"]
#[DbValueStyle = "snake_case"]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = super::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbOrder {
    pub order_id: i64,
    pub user_id: i64,
    pub price: i64,
    pub quantity: f64,
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
