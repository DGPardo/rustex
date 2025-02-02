diesel::table! {
    orders (order_id, user_id) {
        order_id -> Uuid,
        user_id -> Int8,
        price -> Int8,
        quantity -> Float8,
        utc_epoch -> Int8,
        buy_order -> Bool,
    }
}
