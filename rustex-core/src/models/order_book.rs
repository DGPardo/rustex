use std::{collections::BinaryHeap, sync::Mutex};

use crate::{lock, order_matching::MatchOrders};

use super::{
    orders::{BuyOrder, Order, OrderId, SellOrder},
    trade::{Trade, TradeId},
    EpochTime, UserId,
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
    order_counter: Mutex<OrderId>,                      // TODO: Perhaps AtomicU64 is enough?
    trade_counter: Mutex<TradeId>,                      // TODO: Perhaps AtomicU64 is enough?
}

impl OrderBook {
    fn fetch_next_order_id(&self) -> OrderId {
        lock!(self.order_counter).fetch_increment()
    }

    fn fetch_next_trade_id(&self) -> TradeId {
        lock!(self.trade_counter).fetch_increment()
    }

    pub fn insert_buy_order(
        &self,
        user_id: UserId,
        price: u64,
        quantity: f64,
        time: EpochTime,
    ) -> (OrderId, Vec<Trade>) {
        let order_id = self.fetch_next_order_id();
        let order = Order {
            id: order_id,
            user_id,
            price,
            quantity,
            unix_epoch: time,
        };
        let trades = BuyOrder::from(order).match_order(self);
        (order_id, trades)
    }

    pub fn insert_sell_order(
        &self,
        user_id: UserId,
        price: u64,
        quantity: f64,
        time: EpochTime,
    ) -> (OrderId, Vec<Trade>) {
        let order_id = self.fetch_next_order_id();
        let order = Order {
            id: order_id,
            user_id,
            price,
            quantity,
            unix_epoch: time,
        };
        let trades = SellOrder::from(order).match_order(self);
        (order_id, trades)
    }

    pub fn make_trade(
        &self,
        buy_order_id: OrderId,
        sell_order_id: OrderId,
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
