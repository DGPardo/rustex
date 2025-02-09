pub mod db_service;
pub mod match_service;

use db_service::DbServiceClient;
use match_service::MatchServiceClient;

pub(crate) const DEFAULT_MAX_NUMBER_CO_CONNECTIONS: usize = 10_000;
pub(crate) const DEFAULT_ADDRESS: &str = "127.0.0.1"; // Of this microservice

pub use db_service::ADDRESS as DB_RPC_ADDRESS;
pub use match_service::ADDRESS as MATCH_RPC_ADDRESS;

macro_rules! generate_client {
    ($($client_type:ty),* $(,)?) => {
        paste::paste! {
            $(
                pub async fn [< get_ $client_type:snake >]<A: tokio::net::ToSocketAddrs>(addrs: A) -> anyhow::Result<$client_type> {
                    let mut transport = tarpc::serde_transport::tcp::connect(
                        addrs,
                        tarpc::tokio_serde::formats::Json::default
                    );
                    transport.config_mut().max_frame_length(u32::MAX as usize);

                    let client = $client_type::new(tarpc::client::Config::default(), transport.await?).spawn();
                    Ok(client)
                }
            )*
        }
    };
}

generate_client!(MatchServiceClient, DbServiceClient);
