use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use futures_util::future::{ready, Ready};

use crate::{auth::parse_auth_header, auth::Claims};

impl FromRequest for Claims {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let token_data = match parse_auth_header::<Claims>(req.headers()) {
            Ok(token_data) => token_data,
            Err(e) => {
                return ready(Err(e));
            }
        };
        ready(Ok(token_data.claims))
    }
}
