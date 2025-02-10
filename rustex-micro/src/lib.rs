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

#[macro_export]
macro_rules! create_tarpc_server {
    ($address:expr, $max_conns:expr, $server_state:expr) => {{
        let mut listener = tarpc::serde_transport::tcp::listen(
            $address,
            tarpc::tokio_serde::formats::Json::default,
        )
        .await
        .unwrap();

        listener.config_mut().max_frame_length(u32::MAX as usize);

        async fn tokio_spawn(fut: impl Future<Output = ()> + Send + 'static) {
            tokio::spawn(fut);
        }

        listener
            .filter_map(|r| futures::future::ready(r.ok()))
            .map(tarpc::server::BaseChannel::with_defaults)
            .map(|channel| {
                tarpc::server::Channel::execute(channel, $server_state.clone().serve())
                    .for_each(tokio_spawn)
            })
            .buffered($max_conns) // in order
            .for_each(|_| async {})
    }};
}
