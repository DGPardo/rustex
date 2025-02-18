use chrono::{DateTime, Utc};
use diesel::{prelude::*, sql_types::BigInt, AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};

use super::orders::{ExchangeMarket, OrderId};

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
pub struct TradeId(i64);

impl From<i64> for TradeId {
    fn from(value: i64) -> Self {
        TradeId(value)
    }
}

impl From<TradeId> for i64 {
    fn from(value: TradeId) -> Self {
        value.0
    }
}

impl std::ops::Add<i64> for TradeId {
    type Output = TradeId;
    fn add(self, rhs: i64) -> Self::Output {
        TradeId(self.0 + rhs)
    }
}

impl<DB> diesel::serialize::ToSql<BigInt, DB> for TradeId
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

impl<DB> diesel::deserialize::FromSql<BigInt, DB> for TradeId
where
    DB: diesel::backend::Backend,
    i64: diesel::deserialize::FromSql<BigInt, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Ok(TradeId(i64::from_sql(bytes)?))
    }
}

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Debug, PartialEq)]
#[diesel(table_name = crate::db::schema::trades)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Trade {
    pub trade_id: TradeId,
    pub exchange: ExchangeMarket,
    pub buy_order: OrderId,
    pub sell_order: OrderId,
    pub price: i64,
    pub quantity: f64,
    pub created_at: Option<DateTime<Utc>>, // Diesel automatically handles time-zone conversions
}
