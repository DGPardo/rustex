use std::collections::HashMap;

use actix_web::{web, HttpResponse};
use rustex_core::prelude::{ExchangeMarkets, OrderId};
use rustex_errors::RustexError;
use serde::Deserialize;
use serde_json::json;
use tarpc::context::{self, Context};
use tokio::task::JoinSet;

use crate::{api_rest::state::AppState, auth::Claims};

pub async fn get_orders(
    user: Claims,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RustexError> {
    let mut tasks = JoinSet::new();

    for (&market, rpc_client) in state.match_orders.iter() {
        let rpc_client = rpc_client.clone();
        tasks.spawn(async move {
            if let Ok(Ok(orders)) = rpc_client
                .get_user_orders(Context::current(), user.sub)
                .await
            {
                (market, Some(orders))
            } else {
                log::error!("Failed to pull the orders for market: {:?}", market);
                (market, None)
            }
        });
    }

    let orders: HashMap<ExchangeMarkets, Option<Vec<OrderId>>> =
        tasks.join_all().await.into_iter().collect();

    Ok(HttpResponse::Ok().json(orders))
}

#[derive(Deserialize)]
pub struct ClientOrder {
    pub price: i64,
    pub quantity: f64,
    pub exchange: ExchangeMarkets,
}

#[derive(Deserialize)]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(Deserialize)]
pub struct OrderQuery {
    order_type: OrderType,
}

pub async fn insert_order(
    query: web::Query<OrderQuery>,
    order_info: web::Json<ClientOrder>,
    user: Claims,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RustexError> {
    // Using enum impl to route the query
    query
        .order_type
        .insert_new_order(state, order_info, user)
        .await
}

impl OrderType {
    pub async fn insert_new_order(
        &self,
        state: web::Data<AppState>,
        order: web::Json<ClientOrder>,
        user: Claims,
    ) -> Result<HttpResponse, RustexError> {
        macro_rules! insert_order {
            ($fname:ident) => {
                if let Some(match_service) = state.match_orders.get(&order.exchange) {
                    match_service.$fname(context::current(), user.sub, order.price, order.quantity)
                } else {
                    return Err(RustexError::MatchServiceError(
                        "Failed to locate the requested exchange market".into(),
                    ));
                }
            };
        }

        let (order_id, trades) = match self {
            OrderType::Buy => insert_order!(insert_buy_order).await?,
            OrderType::Sell => insert_order!(insert_sell_order).await?,
        }?;

        Ok(HttpResponse::Ok().json(json!({
            "order_id": order_id,
            "trades": trades,
        })))
    }
}

#[derive(Deserialize)]
pub struct OrderStateQuery {
    order_id: OrderId,
    market: ExchangeMarkets,
}

pub async fn get_order_state(
    state: web::Data<AppState>,
    query: web::Query<OrderStateQuery>,
    user: Claims,
) -> Result<HttpResponse, RustexError> {
    let (order_id, market) = (query.order_id, query.market);
    if let Some(market) = state.match_orders.get(&market) {
        let progress = market
            .get_order_progress(Context::current(), user.sub, order_id)
            .await?;
        Ok(HttpResponse::Ok().json(progress))
    } else {
        Err(RustexError::UserFacingError(
            "Requested market exchange is not available in this server".into(),
        ))
    }
}

pub async fn try_delete_order(
    _state: web::Data<AppState>,
    _order_id: web::Path<OrderId>,
    _user: Claims,
) -> Result<HttpResponse, RustexError> {
    unimplemented!()
}
