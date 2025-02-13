use std::{collections::HashMap, str::FromStr};

use anyhow::Context;
use rpc_clients::match_service::MatchServiceClient;
use rustex_core::prelude::ExchangeMarkets;

pub struct AppState {
    pub match_orders: HashMap<ExchangeMarkets, MatchServiceClient>,
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        let markets: Vec<String> = std::env::var("EXCHANGE_MARKETS")
            .map(|s| s.split(",").map(|m| m.to_string()).collect())
            .unwrap_or_default();
        let mut match_orders = HashMap::new();
        for market in markets {
            let market = ExchangeMarkets::from_str(&market)?;
            let rpc_client_address = std::env::var(format!("{:?}_RPC_MATCH_SERVER", &market))
                .context(format!(
                    "Exchange market {:?} has not specified its matching server",
                    &market
                ))
                .unwrap();
            log::info!(
                "Connecting to market {:?} on {:?}",
                market,
                rpc_client_address
            );
            let rpc_client = rpc_clients::get_match_service_client(rpc_client_address)
                .await
                .context("Failed to connect to Market Match Service Client")?;
            match_orders.insert(market, rpc_client);
        }
        Ok(Self { match_orders })
    }
}
