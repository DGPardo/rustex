use chrono::{DateTime, Utc};
use jsonwebtoken as jwt;
use rustex_core::prelude::UserId;
use rustex_errors::RustexError;
use serde::{Deserialize, Serialize};

use super::JWT_SECRET_KEY;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: UserId, // Subject -> user id
    pub exp: u128,   // Expiration timestamp
    pub email: Option<String>,
    pub role: Option<String>,
}

pub fn generate_jwt_token(
    curr_time: DateTime<Utc>,
    user_id: UserId,
    email: Option<String>,
    role: Option<String>,
) -> Result<String, RustexError> {
    let expiration = curr_time.timestamp() as u128 + 3600; // Expiration time (1 hour from now)
    let claims = Claims {
        sub: user_id,
        exp: expiration,
        email,
        role,
    };

    let mut token = "Bearer ".to_string();
    token.push_str(
        jwt::encode(
            &jwt::Header::default(),
            &claims,
            &jwt::EncodingKey::from_secret(JWT_SECRET_KEY.as_ref()),
        )?
        .as_str(),
    );
    Ok(token)
}
