use anyhow::Context;
use rpc_clients::{
    db_service::DbServiceClient, match_service::MatchServiceClient, time_service::TimeServiceClient,
};

pub struct AppState {
    pub _db_rpc_client: DbServiceClient,
    pub time_rpc_client: TimeServiceClient,
    pub match_order_rpc_client: MatchServiceClient,
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            _db_rpc_client: rpc_clients::get_db_service_client(("127.0.0.1", 6666))
                .await
                .context("Failed to connect to DB Service Client")?, // TODO: env vars
            time_rpc_client: rpc_clients::get_time_service_client(("127.0.0.1", 7777))
                .await
                .context("Failed to connect to Time Service Client")?, // TODO: env vars
            match_order_rpc_client: rpc_clients::get_match_service_client(("127.0.0.1", 5555))
                .await
                .context("Failed to connect to Match Service Client")?, // TODO: env vars
        })
    }
}
