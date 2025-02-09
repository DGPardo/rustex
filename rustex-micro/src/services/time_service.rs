use dotenvy::dotenv;

#[tokio::main]
pub async fn main() {
    dotenv().unwrap();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    rpc_clients::time_service::start_service().await
}
