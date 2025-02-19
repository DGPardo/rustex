use actix_web::{web, Scope};

use crate::api_rest::handlers::*;

// No Security Middleware
pub fn get_public_api_service() -> Scope {
    web::scope("/v1/public")
        .route("/health", web::get().to(health::service_health))
        .route("/auth/login", web::post().to(users::login)) // TODO
}

// JWT Middleware-wrapped
pub fn get_protected_api_service() -> Scope {
    web::scope("/v1")
        .route("/health", web::get().to(health::service_health)) // For testing
        .service(
            web::resource("/orders")
                // Lists all orders for a given user
                .route(web::get().to(orders::get_orders))
                // Creates a new order for the given user
                .route(web::post().to(orders::insert_order)),
        )
        .service(
            web::resource("/{exchange_market}/{order_id}")
                .route(web::get().to(orders::get_order_state))
                // Tries to deletes a given order
                .route(web::delete().to(orders::try_delete_order)),
        )
}
