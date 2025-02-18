use std::{
    collections::{BinaryHeap, HashSet},
    ops::Deref,
    sync::{
        atomic::{AtomicI64, Ordering},
        Mutex,
    },
};

use rustex_errors::RustexError;

use super::{
    orders::{ClientOrder, ExchangeMarket, Order},
    trades::TradeId,
    UserId,
};
use crate::models::{
    orders::{BuyOrder, OrderId, SellOrder},
    trades::Trade,
};
use crate::{lock, order_matching::MatchOrders};

/// Book Tracking of orders
///
/// Buying matching logic checks for sell orders
/// with prices less than or equal to the buy price
///
/// Selling matching logic checks for buy orders
/// with prices greater than or equal to the sell price
#[derive(Debug)]
pub struct OrderBook {
    pub(crate) buy_orders: Mutex<BinaryHeap<BuyOrder>>, // Max-heap. Highest price at the root
    pub(crate) sell_orders: Mutex<BinaryHeap<SellOrder>>, // Min-heap. Lowest price at the root
    pending_orders: Mutex<HashSet<OrderId>>,            // Orders being processed
    order_counter: AtomicI64,
    trade_counter: AtomicI64,
    exchange: ExchangeMarket,
}

impl OrderBook {
    pub fn new(exchange: ExchangeMarket) -> Self {
        Self {
            buy_orders: Mutex::new(BinaryHeap::new()),
            sell_orders: Mutex::new(BinaryHeap::new()),
            pending_orders: Mutex::new(HashSet::new()),
            order_counter: AtomicI64::new(0),
            trade_counter: AtomicI64::new(0),
            exchange,
        }
    }

    pub fn from_db(
        last_order: OrderId,
        last_trade: TradeId,
        buy_orders: Vec<BuyOrder>,
        sell_orders: Vec<SellOrder>,
        exchange: ExchangeMarket,
    ) -> Self {
        let pending = buy_orders
            .iter()
            .map(|e| e.0.order_id)
            .chain(sell_orders.iter().map(|e| e.0.order_id))
            .collect::<HashSet<OrderId>>();
        Self {
            buy_orders: Mutex::new(BinaryHeap::from(buy_orders)),
            sell_orders: Mutex::new(BinaryHeap::from(sell_orders)),
            pending_orders: Mutex::new(pending),
            order_counter: AtomicI64::new(last_order.into()),
            trade_counter: AtomicI64::new(last_trade.into()),
            exchange,
        }
    }

    fn fetch_next_order_id(&self) -> OrderId {
        self.order_counter.fetch_add(1, Ordering::Relaxed).into()
    }

    fn fetch_next_trade_id(&self) -> TradeId {
        self.trade_counter.fetch_add(1, Ordering::Relaxed).into()
    }

    pub fn process_order<T: MatchOrders + Deref<Target = Order>>(
        &self,
        order: T,
    ) -> (Vec<Trade>, Vec<OrderId>) {
        lock!(self.pending_orders).insert(order.order_id);
        let (trades, completed_orders) = order.match_order(self);

        let mut pending = lock!(self.pending_orders);
        completed_orders.iter().for_each(|oid| {
            pending.remove(oid);
        });
        (trades, completed_orders)
    }

    pub fn into_order<T: From<Order>>(
        &self,
        client_order: ClientOrder,
        user_id: UserId,
    ) -> Result<T, RustexError> {
        if self.exchange != client_order.exchange {
            return Err(RustexError::OtherInternal(
                "Exchange markets do not match".into(),
            ));
        }
        let order = Order {
            order_id: self.fetch_next_order_id(),
            user_id,
            price: client_order.price,
            quantity: client_order.quantity,
            created_at: None,
            order_type: client_order.order_type,
            exchange: self.exchange,
        };
        Ok(T::from(order))
    }

    pub fn make_trade(
        &self,
        buy_order_id: OrderId,
        sell_order_id: OrderId,
        price: i64,
        quantity: f64,
    ) -> Trade {
        Trade {
            trade_id: self.fetch_next_trade_id(),
            exchange: self.exchange,
            buy_order: buy_order_id,
            sell_order: sell_order_id,
            price,
            quantity,
            created_at: None,
        }
    }
}
