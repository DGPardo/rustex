use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, LazyLock},
};

use actix_web::{web, App, HttpServer};
use state::AppState;

mod handlers;
mod routes;
mod state;

const DEFAULT_ADDRESS: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "5000";

static SERVER_ADDRESS: LazyLock<Box<str>> = LazyLock::new(|| {
    std::env::var("SERVER_ADDRESS")
        .map(|addr| addr.into_boxed_str())
        .unwrap_or_else(|_| DEFAULT_ADDRESS.into())
});

static SERVER_PORT: LazyLock<Box<str>> = LazyLock::new(|| {
    std::env::var("SERVER_PORT")
        .map(|port| port.into_boxed_str())
        .unwrap_or_else(|_| DEFAULT_PORT.into())
});

/// TLS-Enabled HTTP Server as described by the actix-web documentation
/// Reference: https://actix.rs/docs/http2/
/// This function will panics if it cannot create the server
pub async fn launch_http_server() {
    let mut certs_file = BufReader::new(File::open("cert.pem").unwrap());
    let mut key_file = BufReader::new(File::open("key.pem").unwrap());

    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
        .next()
        .unwrap()
        .unwrap();

    // set up TLS config options
    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap();

    let app_state = web::Data::new(AppState::new().await.unwrap());
    let result = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&app_state))
            .service(routes::get_api_service())
    })
    .bind_rustls_0_23(format!("{}:{}", *SERVER_ADDRESS, *SERVER_PORT), tls_config)
    .unwrap()
    .workers(4)
    .run()
    .await;

    if let Err(result) = result {
        log::error!("HttpServer errored: {:?}", result)
    }
}
