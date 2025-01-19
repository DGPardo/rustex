#[cfg(feature = "rest_api")]
mod api_rest;
#[cfg(feature = "socket_api")]
mod api_socket;
mod auth;

#[tokio::main]
async fn main() {
    let mut servers = tokio::task::JoinSet::new();

    #[cfg(feature = "rest_api")]
    servers.spawn(api_rest::launch_http_server());

    #[cfg(feature = "socket_api")]
    servers.spawn(api_socket::launch_socket_server());

    tokio::select! {
        _ = servers.join_all() => {
            // Servers aborted
        }
        _ = tokio::signal::ctrl_c() => {
            // Graceful shutdown logic
        }
    }
}
