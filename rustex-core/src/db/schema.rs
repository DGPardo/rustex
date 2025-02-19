// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "exchangemarket"))]
    pub struct Exchangemarket;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "ordertype"))]
    pub struct Ordertype;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Exchangemarket;

    cancelled_orders (order_id, exchange) {
        order_id -> Int8,
        exchange -> Exchangemarket,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Ordertype;
    use super::sql_types::Exchangemarket;

    orders (order_id, exchange) {
        order_id -> Int8,
        user_id -> Int8,
        price -> Int8,
        quantity -> Float8,
        created_at -> Nullable<Timestamptz>,
        order_type -> Ordertype,
        exchange -> Exchangemarket,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Exchangemarket;

    pending_orders (order_id, exchange) {
        order_id -> Int8,
        exchange -> Exchangemarket,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Exchangemarket;

    trades (trade_id, exchange) {
        trade_id -> Int8,
        exchange -> Exchangemarket,
        buy_order -> Int8,
        sell_order -> Int8,
        price -> Int8,
        quantity -> Float8,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(cancelled_orders, orders, pending_orders, trades,);
