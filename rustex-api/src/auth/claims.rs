use rustex_core::prelude::UserId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: UserId, // Subject -> user id
    pub exp: usize,  // Expiration timestamp
    pub email: Option<String>,
    pub role: Option<String>,
}
