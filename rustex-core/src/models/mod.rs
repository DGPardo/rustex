use diesel::{sql_types::BigInt, AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};
pub mod order_book;
pub mod orders;
pub mod trades;

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
pub struct UserId(i64);

impl From<i64> for UserId {
    fn from(value: i64) -> Self {
        UserId(value)
    }
}

impl From<UserId> for i64 {
    fn from(value: UserId) -> Self {
        value.0
    }
}

impl<DB> diesel::serialize::ToSql<BigInt, DB> for UserId
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

impl<DB> diesel::deserialize::FromSql<BigInt, DB> for UserId
where
    DB: diesel::backend::Backend,
    i64: diesel::deserialize::FromSql<BigInt, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Ok(UserId(i64::from_sql(bytes)?))
    }
}
