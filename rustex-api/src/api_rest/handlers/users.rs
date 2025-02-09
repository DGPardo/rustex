use actix_web::web;
use chrono::Utc;
use rustex_errors::RustexError;
use serde::Deserialize;

use crate::{api_rest::state::AppState, auth};

#[derive(Deserialize)]
pub struct Credentials {
    #[allow(dead_code)]
    username: String,
    #[allow(dead_code)]
    hashed_password: String, // TODO: Salt + Nonce
}

type JwtToken = String;

pub async fn login(
    _state: web::Data<AppState>,
    _credentials: web::Json<Credentials>,
) -> Result<JwtToken, RustexError> {
    // TODO: Use third-party identity provider

    let now = Utc::now();
    let token = auth::generate_jwt_token(now, 0.into(), None, None)?;
    Ok(token)

    // let credentials = credentials.into_inner();

    // // Fetch current time and validate login concurrently
    // let (curr_time_result, valid_login_result) = tokio::join!(
    //     state.time_rpc_client.get_time(context::current()),
    //     state.login_rpc_client.login(
    //         context::current(),
    //         credentials.username,
    //         credentials.hashed_password
    //     )
    // );

    // let curr_time = curr_time_result
    //     .context("Failed to fetch current time")
    //     .map_err(error::ErrorInternalServerError)?
    //     .map_err(error::ErrorInternalServerError)?;

    // let valid_login = valid_login_result
    //     .context("Failed to validate login")
    //     .map_err(error::ErrorUnauthorized)?;

    // if valid_login {
    //     // Generate JWT token
    //     let jwt = auth::generate_jwt_token(curr_time, UserId::from(0), None, None)
    //         .context("Failed to generate JWT token")
    //         .map_err(error::ErrorInternalServerError)?;

    //     Ok(jwt)
    // } else {
    //     Err(error::ErrorUnauthorized("Unauthorized"))
    // }
}
