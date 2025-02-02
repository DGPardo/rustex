use std::{fs::File, io::BufReader, sync::LazyLock};

use actix_web::{web, App, HttpServer};
use anyhow::Context;
use rustls::ServerConfig;
use state::AppState;

use crate::auth::JwtMiddleware;

mod handlers;
mod routes;
mod state;

const DEFAULT_ADDRESS: &str = "127.0.0.1";
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
    let mut certs_file =
        BufReader::new(File::open("../tls_certs/cert.pem").context("Failed to read cert.pem")?);
    let mut key_file =
        BufReader::new(File::open("../tls_certs/key.pem").context("Failed to read key.pem")?);

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
            .app_data(web::Data::clone(&app_state))
            .service(routes::get_public_api_service())
            .service(routes::get_protected_api_service().wrap(JwtMiddleware))
    })
    .bind_rustls_0_23(format!("{}:{}", *SERVER_ADDRESS, *SERVER_PORT), tls_config)
    .unwrap()
    .workers(4)
    .run()
    .await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use std::{future::Future, time::Duration};
    use dotenvy::dotenv;
    use rpc_clients::{match_service, time_service, db_service};
    use super::*;

    async fn with_services(inner: impl Future<Output = ()>) {
        dotenv().unwrap();
        let db_micro_service = tokio::spawn(db_service::start_service());
        let time_micro_service = tokio::spawn(time_service::start_service());
        let match_micro_service = tokio::spawn(match_service::start_service());
        let http_server = tokio::spawn(launch_http_server());

        tokio::time::sleep(Duration::from_secs(1)).await; // init time

        inner.await;

        if http_server.is_finished() {
            println!("Server should still be running")
        }
        if time_micro_service.is_finished() {
            println!("time micro-service should still be running")
        }
        if match_micro_service.is_finished() {
            println!("match micro-service should still be running")
        }
        if db_micro_service.is_finished() {
            println!("db micro-service should still be running")
        }

        http_server.abort();
        match_micro_service.abort();
        time_micro_service.abort();
        db_micro_service.abort();
    }

    #[tokio::test]
    async fn test_public_routes() {
        with_services(async move {
            let api_base = format!("https:://{:?}:{:?}", &SERVER_ADDRESS, &SERVER_PORT);
            let public_ep = format!("{}/v1/public/health", &api_base);
            let public_ep = reqwest::get(public_ep).await;
            assert!(public_ep.is_ok_and(|r| r.status().is_success()));
        })
        .await;
    }

    #[tokio::test]
    async fn test_protected_routes() {
        with_services(async move {
            let api_base = format!("https:://{:?}:{:?}", &SERVER_ADDRESS, &SERVER_PORT);
            let protected_ep = format!("{}/v1/health", &api_base);
            let protected_ep = reqwest::get(protected_ep).await;
            assert!(protected_ep.is_ok_and(|r| r.status().is_client_error()));
        })
        .await;
    }
}
