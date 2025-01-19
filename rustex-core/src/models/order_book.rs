use std::{collections::BinaryHeap, sync::Mutex};

use crate::{lock, order_matching::MatchOrders};

use super::{
    orders::{BuyOrder, Order, SellOrder},
    trade::Trade,
    EpochTime,
};

/// Book Tracking of orders
///
/// Buying matching logic checks for sell orders
/// with prices less than or equal to the buy price
///
/// Selling matching logic checks for buy orders
/// with prices greater than or equal to the sell price
#[derive(Debug, Default)]
pub struct OrderBook {
    pub(crate) buy_orders: Mutex<BinaryHeap<BuyOrder>>, // Max-heap. Highest price at the root
    pub(crate) sell_orders: Mutex<BinaryHeap<SellOrder>>, // Min-heap. Lowest price at the root
    order_counter: Mutex<u128>,
    trade_counter: Mutex<u128>,
}

impl OrderBook {
    fn fetch_next_order_id(&self) -> u128 {
        let mut counter = lock!(self.order_counter);
        *counter += 1;
        *counter
    }

    fn fetch_next_trade_id(&self) -> u128 {
        let mut counter = lock!(self.trade_counter);
        *counter += 1;
        *counter
    }

    pub fn insert_buy_order(
        &self,
        user_id: u64,
        price: u64,
        quantity: f64,
    ) -> Result<(u128, Vec<Trade>), Box<dyn std::error::Error>> {
        let order_id = self.fetch_next_order_id();
        let order = Order {
            id: order_id,
            user_id,
            price,
            quantity,
            unix_epoch: EpochTime::now()?,
        };
        let trades = BuyOrder::from(order).match_order(self);
        Ok((order_id, trades))
    }

    pub fn insert_sell_order(
        &self,
        user_id: u64,
        price: u64,
        quantity: f64,
    ) -> Result<(u128, Vec<Trade>), Box<dyn std::error::Error>> {
        let order_id = self.fetch_next_order_id();
        let order = Order {
            id: order_id,
            user_id,
            price,
            quantity,
            unix_epoch: EpochTime::now()?,
        };
        let trades = SellOrder::from(order).match_order(self);
        Ok((order_id, trades))
    }

    pub fn make_trade(
        &self,
        buy_order_id: u128,
        sell_order_id: u128,
        price: u64,
        quantity: f64,
    ) -> Trade {
        Trade {
            id: self.fetch_next_trade_id(),
            buy_order_id,
            sell_order_id,
            price,
            quantity,
        }
    }
}
