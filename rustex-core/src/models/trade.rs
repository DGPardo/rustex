use serde::{Deserialize, Serialize};

use super::orders::OrderId;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Default, Copy, Clone)]
pub struct TradeId(i64);

impl TradeId {
    pub fn fetch_increment(&mut self) -> TradeId {
        let curr_value = self.0;
        self.0 += 1;
        TradeId(curr_value)
    }
}

impl From<i64> for TradeId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<TradeId> for i64 {
    fn from(value: TradeId) -> Self {
        value.0
    }
}

/// Defines a given trade in the exchange
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Trade {
    pub id: TradeId,
    pub buy_order_id: OrderId,
    pub sell_order_id: OrderId,
    pub price: i64,
    pub quantity: f64,
}

impl Eq for Trade {}

impl PartialEq for Trade {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
