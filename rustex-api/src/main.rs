#[cfg(feature = "rest_api")]
mod api_rest;
#[cfg(feature = "socket_api")]
mod api_socket;
mod auth;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut servers = tokio::task::JoinSet::new();

    #[cfg(feature = "rest_api")]
    servers.spawn(api_rest::launch_http_server());

    #[cfg(feature = "socket_api")]
    servers.spawn(api_socket::launch_socket_server());

    tokio::select! {
        err = servers.join_all() => {
            log::error!("Server aborted {:?}", err);
        }
        _ = tokio::signal::ctrl_c() => {
            log::warn!("Detected Ctrl + C");
        }
    }
}
