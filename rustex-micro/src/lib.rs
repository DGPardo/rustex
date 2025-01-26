pub mod match_service;
pub mod time_service;

use match_service::MatchServiceClient;
use time_service::TimeServiceClient;

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

generate_client!(TimeServiceClient, MatchServiceClient);
