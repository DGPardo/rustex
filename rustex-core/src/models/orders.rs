use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

use super::EpochTime;

#[derive(Debug, PartialEq, Eq)]
pub struct BuyOrder(Order);

#[derive(Debug, PartialEq, Eq)]
pub struct SellOrder(Order);

/// Defines an order (either buy or sell order)
#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    /// This will be unique and always increasing number
    pub(crate) id: u128,
    pub(crate) user_id: u64,
    pub(crate) price: u64, // working with cents
    pub(crate) quantity: f64,
    pub(crate) unix_epoch: EpochTime,
}

impl Eq for Order {}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.unix_epoch == other.unix_epoch
    }
}

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
            .then(self.unix_epoch.cmp(&other.unix_epoch))
    }
}

impl Ord for SellOrder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.price
            .cmp(&other.price)
            .reverse() // Lowest to highest sell price
            .then(self.unix_epoch.cmp(&other.unix_epoch))
    }
}
