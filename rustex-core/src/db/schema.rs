// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "ordertype"))]
    pub struct Ordertype;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Ordertype;

    orders (order_id, user_id) {
        order_id -> Int8,
        user_id -> Int8,
        price -> Int8,
        quantity -> Float8,
        created_at -> Nullable<Timestamptz>,
        order_type -> Ordertype,
    }
}

diesel::table! {
    trades (trade_id) {
        trade_id -> Int8,
        buy_order -> Int8,
        sell_order -> Int8,
        price -> Int8,
        quantity -> Float8,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(orders, trades,);
