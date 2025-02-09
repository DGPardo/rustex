use std::collections::HashMap;

use anyhow::Context;
use rpc_clients::{match_service::MatchServiceClient, MATCH_RPC_ADDRESS};
use rustex_core::prelude::ExchangeMarkets;

pub struct AppState {
    pub match_orders: HashMap<ExchangeMarkets, MatchServiceClient>,
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        let btc_usd = ExchangeMarkets::BTC_USD;
        let rpc_client = rpc_clients::get_match_service_client(MATCH_RPC_ADDRESS.clone())
            .await
            .context("Failed to connect to BTC_USD Match Service Client")?;

        let mut match_orders = HashMap::new();
        match_orders.insert(btc_usd, rpc_client);

        Ok(Self { match_orders })
    }
}
