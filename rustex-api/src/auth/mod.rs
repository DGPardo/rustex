mod claims;

mod extractor;
mod middleware;

use std::sync::LazyLock;

use actix_web::http::header::HeaderMap;
use jsonwebtoken as jwt;
use serde::Deserialize;

pub use claims::{generate_jwt_token, Claims};
pub use middleware::JwtMiddleware;

static JWT_SECRET_KEY: LazyLock<String> = LazyLock::new(|| {
    std::env::var("JWT_SECRET_KEY").expect("Failed to read JWT_SECRET_KEY environment variable")
});

fn parse_auth_header<T>(headers: &HeaderMap) -> Result<jwt::TokenData<T>, actix_web::Error>
where
    T: for<'de> Deserialize<'de>,
{
    let auth_header = headers
        .get("Authorization")
        .and_then(|auth_header| auth_header.to_str().ok())
        .filter(|auth_str| auth_str.starts_with("Bearer "))
        .map(|auth_str| &auth_str[7..]); // Strip "Bearer " prefix

    let token = match auth_header {
        Some(auth_str) => auth_str,
        _ => {
            return Err(actix_web::error::ErrorUnauthorized(
                "Authorization header missing or malformed",
            ));
        }
    };

    match jwt::decode::<T>(
        token,
        &jwt::DecodingKey::from_secret(JWT_SECRET_KEY.as_bytes()),
        &jwt::Validation::default(),
    ) {
        Ok(token_data) => Ok(token_data),
        Err(err) => {
            // Token validation failed
            log::error!("Token validation error: {:?}", err);
            Err(actix_web::error::ErrorUnauthorized("Invalid token"))
        }
    }
}
