use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::orders::{ExchangeMarket, OrderId};

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::db::schema::cancelled_orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CancelledOrder {
    pub order_id: OrderId,
    pub exchange: ExchangeMarket,
    pub created_at: Option<DateTime<Utc>>, // Diesel automatically handles time-zone conversions
}
