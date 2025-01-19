use serde::{Deserialize, Serialize};

/// Defines a given trade in the exchange
#[derive(Serialize, Deserialize, Debug)]
pub struct Trade {
    /// This will be unique and always increasing
    pub(crate) id: u128,
    pub(crate) buy_order_id: u128,
    pub(crate) sell_order_id: u128,
    pub(crate) price: u64,
    pub(crate) quantity: f64,
}

impl Eq for Trade {}

impl PartialEq for Trade {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
