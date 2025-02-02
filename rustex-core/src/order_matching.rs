use std::ops::DerefMut;

use crate::{
    lock,
    models::{
        order_book::OrderBook,
        orders::{BuyOrder, Order, SellOrder},
        trade::Trade,
    },
    prelude::OrderId,
};

pub trait MatchOrders: DerefMut<Target = Order> {
    fn match_order(self, book: &OrderBook) -> (Vec<Trade>, Vec<OrderId>);
}

impl MatchOrders for BuyOrder {
    fn match_order(mut self, book: &OrderBook) -> (Vec<Trade>, Vec<OrderId>) {
        let mut trades = vec![];
        let mut completed_orders = vec![];

        if self.quantity == 0.0 {
            return (trades, completed_orders);
        }

        {
            let mut sell_orders = lock!(book.sell_orders);

            // Sell orders are sorted from lowest to highest in price
            while let Some(mut sell_order) = sell_orders.pop() {
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
                    self.id,
                    sell_order.id,
                    sell_order.price,
                    trade_quantity,
                ));

                // If the sell order still has some quantity
                if sell_order.quantity.abs() > f64::EPSILON {
                    sell_orders.push(sell_order);
                } else {
                    completed_orders.push(sell_order.id);
                }

                if self.quantity.abs() < f64::EPSILON {
                    completed_orders.push(self.id);
                    return (trades, completed_orders);
                }
            }
        } // Release sell_orders lock

        if self.quantity > 0.0 {
            lock!(book.buy_orders).push(self);
        }
        (trades, completed_orders)
    }
}

impl MatchOrders for SellOrder {
    fn match_order(mut self, book: &OrderBook) -> (Vec<Trade>, Vec<OrderId>) {
        let mut trades = vec![];
        let mut completed_orders = vec![];

        if self.quantity == 0.0 {
            return (trades, completed_orders);
        }

        {
            let mut buy_orders = lock!(book.buy_orders);

            // Buy orders are sorted from highest to lowest in price

            while let Some(mut buy_order) = buy_orders.pop() {
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
                    buy_order.id,
                    self.id,
                    buy_order.price,
                    trade_quantity,
                ));

                // If the sell order still has some quantity
                if self.quantity.abs() > f64::EPSILON {
                    buy_orders.push(buy_order);
                } else {
                    completed_orders.push(buy_order.id);
                }

                if self.quantity.abs() < f64::EPSILON {
                    completed_orders.push(self.id);
                    return (trades, completed_orders);
                }
            }
        } // Release buy_orders lock

        if self.quantity > 0.0 {
            lock!(book.sell_orders).push(self);
        }

        (trades, completed_orders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EpochTime;

    #[test]
    fn test_successful_match() {
        let book = OrderBook::default();

        let now = EpochTime::now().unwrap();
        let (order_id, trades) = book.insert_sell_order(123.into(), 50, 10.0, now);
        assert_eq!(order_id, 0.into());
        assert!(trades.is_empty());

        let (order_id, trades) = book.insert_sell_order(456.into(), 45, 5.0, now);
        assert_eq!(order_id, 1.into());
        assert!(trades.is_empty());

        let (order_id, trades) = book.insert_buy_order(2.into(), 50, 8.0, now);
        assert_eq!(order_id, 2.into());

        assert_eq!(
            trades,
            vec![
                Trade {
                    id: 0.into(),
                    buy_order_id: 2.into(),
                    sell_order_id: 1.into(),
                    price: 45,
                    quantity: 5.0
                },
                Trade {
                    id: 1.into(),
                    buy_order_id: 2.into(),
                    sell_order_id: 0.into(),
                    price: 50,
                    quantity: 3.0
                },
            ]
        );

        let computed_sell_order = lock!(book.sell_orders).pop().unwrap();
        assert_eq!(computed_sell_order.id, 0.into());
        assert_eq!(computed_sell_order.price, 50);
        assert!((computed_sell_order.quantity - 7.0).abs() < f64::EPSILON);
        assert_eq!(computed_sell_order.user_id, 123.into());
    }
}
