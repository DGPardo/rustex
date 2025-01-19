use rustex_core::prelude::UserId;

pub struct AuthedUser {
    user_id: UserId,
    email_address: Option<String>,
}

// TODO: Implement Actix-web extractor and middleware
