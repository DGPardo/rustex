use serde::{Deserialize, Serialize};

use super::orders::OrderId;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Default, Copy, Clone)]
pub struct TradeId(u128);

impl TradeId {
    pub fn fetch_increment(&mut self) -> TradeId {
        let curr_value = self.0;
        self.0 += 1;
        TradeId(curr_value)
    }
}

impl From<u128> for TradeId {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

/// Defines a given trade in the exchange
#[derive(Serialize, Deserialize, Debug)]
pub struct Trade {
    /// This will be unique and always increasing
    pub(crate) id: TradeId,
    pub(crate) buy_order_id: OrderId,
    pub(crate) sell_order_id: OrderId,
    pub(crate) price: i64,
    pub(crate) quantity: f64,
}

impl Eq for Trade {}

impl PartialEq for Trade {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
