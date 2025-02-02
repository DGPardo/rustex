use crate::{api_rest::state::AppState, auth::Claims};
use actix_web::{web, HttpResponse};
use rustex_errors::RustexError;
use serde::Deserialize;
use serde_json::json;
use tarpc::context;

#[derive(Deserialize)]
pub struct ClientOrder {
    pub price: i64,
    pub quantity: f64,
}

pub async fn insert_buy_order(
    state: web::Data<AppState>,
    order: web::Json<ClientOrder>,
    user: Claims,
) -> Result<HttpResponse, RustexError> {
    OrderType::Buy.insert_new_order(state, order, user).await
}

pub async fn insert_sell_order(
    state: web::Data<AppState>,
    order: web::Json<ClientOrder>,
    user: Claims,
) -> Result<HttpResponse, RustexError> {
    OrderType::Sell.insert_new_order(state, order, user).await
}

enum OrderType {
    Buy,
    Sell,
}

impl OrderType {
    pub async fn insert_new_order(
        &self,
        state: web::Data<AppState>,
        order: web::Json<ClientOrder>,
        user: Claims,
    ) -> Result<HttpResponse, RustexError> {
        let time = state.time_rpc_client.get_time(context::current()).await??;

        macro_rules! insert_order {
            ($fname:ident) => {
                state
                    .match_order_rpc_client
                    .$fname(
                        context::current(),
                        user.sub,
                        order.price,
                        order.quantity,
                        time,
                    )
                    .await
            };
        }

        let (order_id, trades) = match self {
            OrderType::Buy => insert_order!(insert_buy_order),
            OrderType::Sell => insert_order!(insert_sell_order),
        }?;

        Ok(HttpResponse::Ok().json(json!({
            "order_id": order_id,
            "trades": trades,
        })))
    }
}

pub async fn get_order_state() -> HttpResponse {
    HttpResponse::Ok().finish()
}
