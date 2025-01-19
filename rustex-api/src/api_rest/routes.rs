use crate::api_rest::handlers::*;
use actix_web::{web, Scope};

pub fn get_api_service() -> Scope {
    web::scope("/v1")
        .route("/health", web::get().to(health::service_health))
        .route("/orders", web::post().to(orders::insert_new_order))
        .route("/orders/{order_id}", web::get().to(orders::get_order_state))

    // .route("/users", web::post().to(handle_new_order))
    // .route("/users", web::get().to(handle_fetch_orders))
}
