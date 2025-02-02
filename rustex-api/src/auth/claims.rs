use rustex_core::prelude::UserId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: UserId, // Subject -> user id
    pub exp: u128,   // Expiration timestamp
    pub email: Option<String>,
    pub role: Option<String>,
}

// pub fn generate_jwt_token(
//     curr_time: EpochTime,
//     user_id: UserId,
//     email: Option<String>,
//     role: Option<String>,
// ) -> anyhow::Result<String> {
//     let expiration = curr_time.into_inner() + 3600; // Expiration time (1 hour from now)

//     let claims = Claims {
//         sub: user_id,
//         exp: expiration,
//         email,
//         role,
//     };

//     Ok(jwt::encode(
//         &jwt::Header::default(),
//         &claims,
//         &jwt::EncodingKey::from_secret(JWT_SECRET_KEY.as_ref()),
//     )?)
// }
