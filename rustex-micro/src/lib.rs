pub mod match_service;
pub mod time_service;

macro_rules! generate_client {
    ($client_type:ty, $name:ident) => {
        paste::item! {
            pub async fn [< $name >]<A: tokio::net::ToSocketAddrs>(addrs: A) -> Result<$client_type, Box<dyn std::error::Error>> {
                let mut transport = tarpc::serde_transport::tcp::connect(addrs, tarpc::tokio_serde::formats::Json::default);
                transport.config_mut().max_frame_length(u32::MAX as usize);

                let client = $client_type::new(tarpc::client::Config::default(), transport.await?).spawn();
                Ok(client)
            }
        }
    };
}

generate_client!(time_service::TimeServiceClient, get_time_service_client);
generate_client!(match_service::MatchServiceClient, get_match_orders_client);
