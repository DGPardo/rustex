use crate::{api_rest::state::AppState, auth::Claims};
use actix_web::{http::StatusCode, web, HttpResponse, HttpResponseBuilder};
use serde::Deserialize;
use serde_json::json;
use tarpc::context;

#[derive(Deserialize)]
pub struct ClientOrder {
    pub price: u64,
    pub quantity: f64,
}

pub async fn insert_buy_order(
    state: web::Data<AppState>,
    order: web::Json<ClientOrder>,
    user: Claims,
) -> HttpResponse {
    OrderType::Buy.insert_new_order(state, order, user).await
}

pub async fn insert_sell_order(
    state: web::Data<AppState>,
    order: web::Json<ClientOrder>,
    user: Claims,
) -> HttpResponse {
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
    ) -> HttpResponse {
        let time = match state.time_rpc_client.get_time(context::current()).await {
            Ok(Ok(time)) => time,
            _ => return HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).finish(),
        };

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

        let insertion = match self {
            OrderType::Buy => insert_order!(insert_buy_order),
            OrderType::Sell => insert_order!(insert_sell_order),
        };

        match insertion {
            Ok((order_id, trades)) => HttpResponse::Ok().json(json!({
                "order_id": order_id,
                "trades": trades,
            })),
            Err(e) => {
                HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).body(e.to_string())
            }
        }
    }
}

pub async fn get_order_state() -> HttpResponse {
    HttpResponse::Ok().finish()
}
