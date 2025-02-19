use actix_web::{web, HttpResponse};
use hashbrown::HashMap;
use rustex_core::prelude::{ClientOrder, ExchangeMarket, OrderId};
use rustex_errors::RustexError;
use serde::Deserialize;
use tarpc::context::Context;
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

    let orders: HashMap<ExchangeMarket, Option<Vec<OrderId>>> =
        tasks.join_all().await.into_iter().collect();

    Ok(HttpResponse::Ok().json(orders))
}

pub async fn insert_order(
    order_info: web::Json<ClientOrder>,
    user: Claims,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RustexError> {
    if let Some(match_service) = state.match_orders.get(&order_info.exchange) {
        let order_id = match_service
            .insert_order(Context::current(), user.sub, order_info.into_inner())
            .await??;
        Ok(HttpResponse::Ok().json(order_id))
    } else {
        Err(RustexError::MatchServiceError(
            "Exchange market is not available in this server".into(),
        ))
    }
}

#[derive(Deserialize)]
pub struct OrderStateQuery {
    order_id: OrderId,
    market: ExchangeMarket,
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
