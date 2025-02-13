use std::{fs::File, io::BufReader, sync::LazyLock};

use actix_web::{middleware::Logger, web, App, HttpServer};
use anyhow::Context;
use rustls::ServerConfig;
use state::AppState;

use crate::auth::JwtMiddleware;

mod handlers;
mod routes;
mod state;

const DEFAULT_ADDRESS: &str = "0.0.0.0";
const DEFAULT_PORT: &str = "5000";

static SERVER_ADDRESS: LazyLock<Box<str>> = LazyLock::new(|| {
    std::env::var("SERVER_ADDRESS")
        .map(|addr| addr.into())
        .unwrap_or_else(|_| DEFAULT_ADDRESS.into())
});

static SERVER_PORT: LazyLock<Box<str>> = LazyLock::new(|| {
    std::env::var("SERVER_PORT")
        .map(|port| port.into())
        .unwrap_or_else(|_| DEFAULT_PORT.into())
});

fn get_tls_config() -> anyhow::Result<ServerConfig> {
    let mut certs_file = BufReader::new(
        File::open(
            std::env::var("TLS_CERT_PATH")
                .context("TLS_CERT_PATH environment variable is not set")?,
        )
        .context("Failed to read cert.pem")?,
    );
    let mut key_file = BufReader::new(
        File::open(
            std::env::var("TLS_KEY_PATH")
                .context("TLS_KEY_PATH environment variable is not set")?,
        )
        .context("Failed to read key.pem")?,
    );

    let tls_certs = rustls_pemfile::certs(&mut certs_file).collect::<Result<Vec<_>, _>>()?;

    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut key_file)
        .map(|key| key.map(rustls::pki_types::PrivateKeyDer::Pkcs8))
        .collect::<Result<Vec<_>, _>>()?;

    if keys.is_empty() {
        return Err(anyhow::Error::msg("Failed to parse pkcs8 key"));
    }

    // set up TLS config options
    let tls_key = keys.remove(0);
    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, tls_key)?;
    Ok(tls_config)
}

/// TLS-Enabled HTTP Server as described by the actix-web documentation
/// Reference: https://actix.rs/docs/http2/
/// This function will panics if it cannot create the server
pub async fn launch_http_server() -> anyhow::Result<()> {
    let tls_config = get_tls_config()?;
    let app_state = web::Data::new(AppState::new().await?);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::clone(&app_state))
            .service(routes::get_public_api_service())
            .service(routes::get_protected_api_service().wrap(JwtMiddleware))
        // .service(routes::get_protected_api_service())
    })
    .bind_rustls_0_23(format!("{}:{}", *SERVER_ADDRESS, *SERVER_PORT), tls_config)
    // .bind(format!("{}:{}", *SERVER_ADDRESS, *SERVER_PORT))
    .unwrap()
    .workers(4)
    .run()
    .await?;

    Ok(())
}
