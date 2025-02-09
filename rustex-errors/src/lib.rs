use std::{
    fmt::{Debug, Display},
    time::SystemTimeError,
};

use actix_web::{http::StatusCode, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum RustexError {
    InternalServerError(InternalServerError),
    UserFacingError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InternalServerError {
    DbServiceError(RustexInternalError),
    TimeServiceError(RustexInternalError),
    MatchServiceError(RustexInternalError),
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

#[macro_export]
macro_rules! match_error {
    ($msg:expr) => {
        rustex_errors::RustexError::InternalServerError(
            rustex_errors::InternalServerError::MatchServiceError(
                rustex_errors::RustexInternalError::from($msg),
            ),
        )
    };
}

impl Display for RustexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            RustexError::InternalServerError(_) => {
                write!(f, "Internal Server Error")
            }
            RustexError::UserFacingError(e) => {
                write!(f, "User Error: {}", e)
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
            RustexError::InternalServerError(e) => {
                log::error!("[INTERNAL SERVER ERROR]: {:?}", e);
                HttpResponse::build(self.status_code()).body("Internal Server Error".to_string())
            }
        }
    }
    fn status_code(&self) -> StatusCode {
        match self {
            RustexError::UserFacingError(_) => StatusCode::BAD_REQUEST,
            RustexError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// Implementation on Foreign Data Types
impl From<SystemTimeError> for RustexError {
    fn from(value: SystemTimeError) -> Self {
        RustexError::InternalServerError(InternalServerError::TimeServiceError(
            RustexInternalError::from(format!("SystemTimeError:: {:?}", value)),
        ))
    }
}

impl From<tarpc::client::RpcError> for RustexError {
    fn from(value: tarpc::client::RpcError) -> Self {
        RustexError::InternalServerError(InternalServerError::TimeServiceError(
            RustexInternalError::from(format!("tarpc::client::RpcError:: {:?}", value)),
        ))
    }
}

impl From<diesel::ConnectionError> for RustexError {
    fn from(value: diesel::ConnectionError) -> Self {
        RustexError::InternalServerError(InternalServerError::DbServiceError(
            RustexInternalError::from(format!("diesel::ConnectionError:: {:?}", value)),
        ))
    }
}

impl From<diesel_async::pooled_connection::deadpool::BuildError> for RustexError {
    fn from(value: diesel_async::pooled_connection::deadpool::BuildError) -> Self {
        RustexError::InternalServerError(InternalServerError::DbServiceError(
            RustexInternalError::from(format!(
                "diesel_async::pooled_connection::deadpool::BuildError:: {:?}",
                value
            )),
        ))
    }
}

impl From<diesel_async::pooled_connection::deadpool::PoolError> for RustexError {
    fn from(value: diesel_async::pooled_connection::deadpool::PoolError) -> Self {
        RustexError::InternalServerError(InternalServerError::DbServiceError(
            RustexInternalError::from(format!(
                "diesel_async::pooled_connection::deadpool::PoolError:: {:?}",
                value
            )),
        ))
    }
}

impl From<diesel::result::Error> for RustexError {
    fn from(value: diesel::result::Error) -> Self {
        RustexError::InternalServerError(InternalServerError::DbServiceError(
            RustexInternalError::from(format!("diesel::result::Error:: {:?}", value)),
        ))
    }
}

impl From<tokio::task::JoinError> for RustexError {
    fn from(value: tokio::task::JoinError) -> Self {
        RustexError::InternalServerError(InternalServerError::DbServiceError(
            RustexInternalError::from(format!("tokio::task::JoinError:: {:?}", value)),
        ))
    }
}
