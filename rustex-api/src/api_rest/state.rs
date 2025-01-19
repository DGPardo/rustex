use rpc_clients::{match_service::MatchServiceClient, time_service::TimeServiceClient};

pub struct AppState {
    pub time_rpc_client: TimeServiceClient,
    pub match_order_rpc_client: MatchServiceClient,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            time_rpc_client: rpc_clients::get_time_service_client(("0.0.0.0", 7777)).await?, // TODO: env vars
            match_order_rpc_client: rpc_clients::get_match_orders_client(("0.0.0.0", 5555)).await?, // TODO: env vars
        })
    }
}
