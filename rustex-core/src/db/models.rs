use diesel::prelude::*;
use uuid::Uuid;

use crate::prelude::{BuyOrder, SellOrder};

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = super::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbOrder {
    pub order_id: Uuid,
    pub user_id: i64,
    pub price: i64,
    pub quantity: f64,
    pub utc_epoch: i64,
    pub buy_order: bool,
}

macro_rules! into_db_order {
    ($order_id:ident, $order:ident, $buy: expr) => {{
        let nanos: u128 = $order.unix_epoch.into_inner();
        if nanos > i64::MAX as u128 {
            panic!("Son, take care of this code.")
        }
        DbOrder {
            $order_id,
            user_id: $order.user_id.into_inner(),
            price: $order.price,
            quantity: $order.quantity,
            utc_epoch: nanos as i64,
            buy_order: $buy,
        }
    }};
}

impl DbOrder {
    pub fn from_sell_order(order_id: Uuid, sell_order: SellOrder) -> Self {
        into_db_order!(order_id, sell_order, false)
    }

    pub fn from_buy_order(order_id: Uuid, buy_order: BuyOrder) -> Self {
        into_db_order!(order_id, buy_order, true)
    }
}
