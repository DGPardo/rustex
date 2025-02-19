use std::{ops::DerefMut, sync::MutexGuard};

use hashbrown::HashSet;

use crate::{
    lock,
    models::{
        order_book::OrderBook,
        orders::{BuyOrder, Order, SellOrder},
        trades::Trade,
    },
    prelude::OrderId,
};

pub trait MatchOrders: DerefMut<Target = Order> {
    fn match_order(
        self,
        book: &OrderBook,
        pending_orders: MutexGuard<HashSet<OrderId>>,
    ) -> (Vec<Trade>, Vec<OrderId>);
}

macro_rules! complete_order {
    ($order_id:expr, $completed:ident, $pending:ident) => {
        $completed.push($order_id);
        $pending.remove(&$order_id);
    };
}

impl MatchOrders for BuyOrder {
    fn match_order(
        mut self,
        book: &OrderBook,
        mut pending_orders: MutexGuard<HashSet<OrderId>>,
    ) -> (Vec<Trade>, Vec<OrderId>) {
        let mut trades = vec![];
        let mut completed_orders = vec![];

        if self.quantity == 0.0 {
            return (trades, completed_orders);
        }

        {
            let mut sell_orders = lock!(book.sell_orders);

            // Sell orders are sorted from lowest to highest in price
            while let Some(mut sell_order) = sell_orders.pop() {
                if !pending_orders.contains(&sell_order.order_id) {
                    continue;
                }
                if sell_order.price > self.price {
                    // No match. The best sell price (lowest price)
                    // Exceeds the bid price (which is too low)
                    sell_orders.push(sell_order); // TODO: avoid pop() and push()
                    break;
                }

                // Compute trade amount and update remainders
                let trade_quantity = sell_order.quantity.min(self.quantity);
                sell_order.quantity -= trade_quantity;
                self.quantity -= trade_quantity;

                // Record the trade
                trades.push(book.make_trade(
                    self.order_id,
                    sell_order.order_id,
                    sell_order.price,
                    trade_quantity,
                ));

                // If the sell order still has some quantity
                if sell_order.quantity.abs() > f64::EPSILON {
                    sell_orders.push(sell_order);
                } else {
                    complete_order!(sell_order.order_id, completed_orders, pending_orders);
                }

                if self.quantity.abs() <= f64::EPSILON {
                    complete_order!(sell_order.order_id, completed_orders, pending_orders);
                    return (trades, completed_orders);
                }
            }
        } // Release sell_orders lock

        if self.quantity > f64::EPSILON {
            lock!(book.buy_orders).push(self);
        }
        (trades, completed_orders)
    }
}

impl MatchOrders for SellOrder {
    fn match_order(
        mut self,
        book: &OrderBook,
        mut pending_orders: MutexGuard<HashSet<OrderId>>,
    ) -> (Vec<Trade>, Vec<OrderId>) {
        let mut trades = vec![];
        let mut completed_orders = vec![];

        if self.quantity == 0.0 {
            return (trades, completed_orders);
        }

        {
            let mut buy_orders = lock!(book.buy_orders);

            // Buy orders are sorted from highest to lowest in price

            while let Some(mut buy_order) = buy_orders.pop() {
                if !pending_orders.contains(&buy_order.order_id) {
                    continue;
                }
                if self.price > buy_order.price {
                    // No match. The best buy price (highest) exceeds
                    // the ask price (which is too high).
                    buy_orders.push(buy_order); // TODO: avoid pop() and push()
                    break;
                }

                // Compute trade amount and update remainders
                let trade_quantity = self.quantity.min(buy_order.quantity);
                self.quantity -= trade_quantity;
                buy_order.quantity -= trade_quantity;

                // Record the trade
                trades.push(book.make_trade(
                    buy_order.order_id,
                    self.order_id,
                    buy_order.price,
                    trade_quantity,
                ));

                // If the sell order still has some quantity
                if buy_order.quantity.abs() > f64::EPSILON {
                    buy_orders.push(buy_order);
                } else {
                    complete_order!(buy_order.order_id, completed_orders, pending_orders);
                }

                if self.quantity.abs() <= f64::EPSILON {
                    complete_order!(self.order_id, completed_orders, pending_orders);
                    return (trades, completed_orders);
                }
            }
        } // Release buy_orders lock

        if self.quantity > f64::EPSILON {
            lock!(book.sell_orders).push(self);
        }

        (trades, completed_orders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::orders::{ClientOrder, ExchangeMarket, OrderType};

    #[test]
    fn test_successful_match() {
        let book = OrderBook::new(ExchangeMarket::BTC_EUR);
        let sell1 = ClientOrder {
            price: 50,
            quantity: 10.0,
            exchange: ExchangeMarket::BTC_EUR,
            order_type: OrderType::Sell,
        };
        let sell2 = ClientOrder {
            price: 45,
            quantity: 5.0,
            exchange: ExchangeMarket::BTC_EUR,
            order_type: OrderType::Sell,
        };
        let buy1 = ClientOrder {
            price: 50,
            quantity: 8.0,
            exchange: ExchangeMarket::BTC_EUR,
            order_type: OrderType::Buy,
        };
        let order: SellOrder = book.into_order(sell1, 123.into()).unwrap();
        assert_eq!(order.order_id, 0.into());
        let pending = std::sync::Mutex::new(HashSet::from([order.order_id]));
        let pending_guard = pending.lock().unwrap();
        let (trades, _completed_orders) = order.match_order(&book, pending_guard);
        assert!(trades.is_empty());

        let order: SellOrder = book.into_order(sell2, 456.into()).unwrap();
        assert_eq!(order.order_id, 1.into());
        let pending = std::sync::Mutex::new(HashSet::from([order.order_id]));
        let pending_guard = pending.lock().unwrap();
        let (trades, _completed_orders) = order.match_order(&book, pending_guard);
        assert!(trades.is_empty());

        let order: BuyOrder = book.into_order(buy1, 2.into()).unwrap();
        assert_eq!(order.order_id, 2.into());
        let pending = std::sync::Mutex::new(HashSet::from([order.order_id]));
        let pending_guard = pending.lock().unwrap();
        let (trades, _completed_orders) = order.match_order(&book, pending_guard);

        assert_eq!(
            trades,
            vec![
                Trade {
                    trade_id: 0.into(),
                    buy_order: 2.into(),
                    sell_order: 1.into(),
                    price: 45,
                    quantity: 5.0,
                    exchange: ExchangeMarket::BTC_EUR,
                    created_at: None,
                },
                Trade {
                    trade_id: 1.into(),
                    buy_order: 2.into(),
                    sell_order: 0.into(),
                    price: 50,
                    quantity: 3.0,
                    exchange: ExchangeMarket::BTC_EUR,
                    created_at: None,
                },
            ]
        );

        let computed_sell_order = lock!(book.sell_orders).pop().unwrap();
        assert_eq!(computed_sell_order.order_id, 0.into());
        assert_eq!(computed_sell_order.price, 50);
        assert!((computed_sell_order.quantity - 7.0).abs() < f64::EPSILON);
        assert_eq!(computed_sell_order.user_id, 123.into());
    }
}
