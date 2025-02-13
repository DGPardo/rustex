use std::{
    fmt::{Debug, Display},
    time::SystemTimeError,
};

use actix_web::{http::StatusCode, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum RustexError {
    UserFacingError(String),
    AuthorizationError(RustexInternalError),
    DbServiceError(RustexInternalError),
    MatchServiceError(RustexInternalError),
    OtherInternal(RustexInternalError),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RustexInternalError {
    msg: Box<str>,
}

impl<T: AsRef<str>> From<T> for RustexInternalError {
    fn from(value: T) -> Self {
        Self {
            msg: value.as_ref().into(),
        }
    }
}

impl Display for RustexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            RustexError::UserFacingError(e) => {
                write!(f, "User Error: {}", e)
            }
            RustexError::AuthorizationError(_) => {
                write!(f, "AUTH Internal Server Error")
            }
            RustexError::DbServiceError(_) => {
                write!(f, "DB Internal Server Error")
            }
            RustexError::MatchServiceError(_) => {
                write!(f, "MATCH Internal Server Error")
            }
            RustexError::OtherInternal(_) => {
                write!(f, "OTHER Internal Server Error")
            }
        }
    }
}

impl std::error::Error for RustexError {}

impl actix_web::ResponseError for RustexError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match self {
            RustexError::UserFacingError(e) => {
                HttpResponse::build(self.status_code()).body(e.to_owned())
            }
            RustexError::AuthorizationError(e) => {
                log::error!("[AUTH INTERNAL SERVER ERROR]: {:?}", e);
                HttpResponse::build(self.status_code())
                    .body("AUTH Internal Server Error".to_string())
            }
            RustexError::DbServiceError(e) => {
                log::error!("[DB INTERNAL SERVER ERROR]: {:?}", e);
                HttpResponse::build(self.status_code()).body("DB Internal Server Error".to_string())
            }
            RustexError::MatchServiceError(e) => {
                log::error!("[MATCH INTERNAL SERVER ERROR]: {:?}", e);
                HttpResponse::build(self.status_code())
                    .body("MATCH Internal Server Error".to_string())
            }
            RustexError::OtherInternal(e) => {
                log::error!("[INTERNAL SERVER ERROR]: {:?}", e);
                HttpResponse::build(self.status_code()).body("Internal Server Error".to_string())
            }
        }
    }
    fn status_code(&self) -> StatusCode {
        match self {
            RustexError::UserFacingError(_) => StatusCode::BAD_REQUEST,
            RustexError::AuthorizationError(_) => StatusCode::UNAUTHORIZED,
            RustexError::DbServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            RustexError::MatchServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            RustexError::OtherInternal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

macro_rules! impl_from_err {
    ($( ($err_type:ty, $variant:ident) ),* ) => {
        $(
            impl From<$err_type> for RustexError {
                fn from(value: $err_type) -> Self {
                    RustexError::$variant(RustexInternalError::from(value.to_string()))
                }
            }
        )*
    };
}

impl_from_err!(
    (SystemTimeError, OtherInternal),
    (tarpc::client::RpcError, OtherInternal),
    (diesel::ConnectionError, DbServiceError),
    (
        diesel_async::pooled_connection::deadpool::BuildError,
        DbServiceError
    ),
    (
        diesel_async::pooled_connection::deadpool::PoolError,
        DbServiceError
    ),
    (diesel::result::Error, DbServiceError),
    (tokio::task::JoinError, OtherInternal),
    (jsonwebtoken::errors::Error, AuthorizationError),
    (anyhow::Error, OtherInternal)
);
