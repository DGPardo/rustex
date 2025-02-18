use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
};

use chrono::{DateTime, Utc};
use diesel::{prelude::*, sql_types::BigInt, AsExpression, FromSqlRow};
use diesel_derive_enum::DbEnum;
use rustex_errors::RustexError;
use serde::{Deserialize, Serialize};

use super::UserId;

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Default,
    Clone,
    Copy,
    Hash,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct OrderId(i64);

impl From<i64> for OrderId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<OrderId> for i64 {
    fn from(value: OrderId) -> Self {
        value.0
    }
}

impl std::ops::Add<i64> for OrderId {
    type Output = OrderId;
    fn add(self, rhs: i64) -> Self::Output {
        OrderId(self.0 + rhs)
    }
}

impl<DB> diesel::serialize::ToSql<BigInt, DB> for OrderId
where
    DB: diesel::backend::Backend,
    i64: diesel::serialize::ToSql<BigInt, DB>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, DB>,
    ) -> diesel::serialize::Result {
        self.0.to_sql(out)
    }
}

impl<DB> diesel::deserialize::FromSql<BigInt, DB> for OrderId
where
    DB: diesel::backend::Backend,
    i64: diesel::deserialize::FromSql<BigInt, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Ok(OrderId(i64::from_sql(bytes)?))
    }
}

#[derive(DbEnum, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[ExistingTypePath = "crate::db::schema::sql_types::Ordertype"]
#[DbValueStyle = "snake_case"]
#[serde(rename_all = "camelCase")]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(DbEnum, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[ExistingTypePath = "crate::db::schema::sql_types::Exchangemarket"]
#[DbValueStyle = "snake_case"]
#[allow(non_camel_case_types)]
pub enum ExchangeMarket {
    BTC_USD,
    BTC_GBP,
    BTC_EUR,
}

impl FromStr for ExchangeMarket {
    type Err = RustexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "BTC_USD" => Ok(ExchangeMarket::BTC_USD),
            "BTC_GBP" => Ok(ExchangeMarket::BTC_GBP),
            "BTC_EUR" => Ok(ExchangeMarket::BTC_EUR),
            _ => Err(RustexError::UserFacingError(format!(
                "{s} is not a valid Exchange Marker"
            ))),
        }
    }
}

impl ExchangeMarket {
    pub fn from_env() -> Result<Self, RustexError> {
        std::env::var("EXCHANGE_MARKET")
            .map(|env_var| ExchangeMarket::from_str(&env_var))
            .map_err(|_| {
                RustexError::UserFacingError(
                    "EXCHANGE_MARKET environment variable is undefined".into(),
                )
            })?
    }
}

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Debug, Clone, Copy)]
#[diesel(table_name = crate::db::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Order {
    pub order_id: OrderId,
    pub user_id: UserId,
    pub price: i64,
    pub quantity: f64,
    pub created_at: Option<DateTime<Utc>>, // Diesel automatically handles time-zone conversions
    pub order_type: OrderType,
    pub exchange: ExchangeMarket,
}

impl Eq for Order {}
impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        (self.order_id == other.order_id) & (self.exchange == other.exchange)
    }
}

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::db::schema::pending_orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PendingOrder {
    pub order_id: OrderId,
    pub exchange: ExchangeMarket,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Copy, Clone)]
pub struct BuyOrder(pub Order);

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Copy, Clone)]
pub struct SellOrder(pub Order);

macro_rules! implement_order_traits {
    ($($order:ident), *) => {
        $(
            impl Deref for $order {
                type Target = Order;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl DerefMut for $order {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }

            impl PartialOrd for $order {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }

            impl From<Order> for $order {
                fn from(order: Order) -> Self {
                    $order(order)
                }
            }
        )*
    };
}

implement_order_traits!(BuyOrder, SellOrder);

impl Ord for BuyOrder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.price
            .cmp(&other.price) // Highest to lowest buy price
            .then(self.0.order_id.cmp(&other.0.order_id))
    }
}

impl Ord for SellOrder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.price
            .cmp(&other.price)
            .reverse() // Lowest to highest sell price
            .then(self.0.order_id.cmp(&other.0.order_id))
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct ClientOrder {
    pub price: i64,
    pub quantity: f64,
    pub exchange: ExchangeMarket,
    pub order_type: OrderType,
}
