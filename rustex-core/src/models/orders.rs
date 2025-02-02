use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

use super::{EpochTime, UserId};

#[derive(
    Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Default, Clone, Copy, Hash,
)]
pub struct OrderId(i64);

impl OrderId {
    pub fn fetch_increment(&mut self) -> OrderId {
        let curr_value = self.0;
        self.0 += 1;
        OrderId(curr_value)
    }
}

impl From<i64> for OrderId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct BuyOrder(Order);

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SellOrder(Order);

/// Defines an order (either buy or sell order)
#[derive(Debug, Deserialize, Serialize)]
pub struct Order {
    /// This will be unique and always increasing number
    pub id: OrderId,
    pub user_id: UserId,
    pub price: i64, // working with cents
    pub quantity: f64,
    pub unix_epoch: EpochTime,
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
