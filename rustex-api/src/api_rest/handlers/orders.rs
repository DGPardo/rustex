use crate::api_rest::state::AppState;
use actix_web::{http::StatusCode, web, HttpResponse, HttpResponseBuilder};
use rustex_core::prelude::Order;
use serde_json::json;
use tarpc::context;

pub async fn insert_new_order(
    state: web::Data<AppState>,
    order: web::Json<Order>,
    user: AuthedUser,
) -> HttpResponse {
    let time = match state.time_rpc_client.get_time(context::current()).await {
        Ok(Ok(time)) => time,
        _ => return HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).finish(),
    };

    let (order_id, trades) = state
        .match_order_rpc_client
        .insert_buy_order(context::current(), user_id, price, quantity, time)
        .await
        .unwrap();

    HttpResponse::Ok().json(json!({
        "order_id": order_id,
        "trades": trades,
    }))
}

pub async fn get_order_state() -> HttpResponse {
    HttpResponse::Ok().finish()
}
