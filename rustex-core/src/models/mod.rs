use std::time::{SystemTime, SystemTimeError};

use serde::{Deserialize, Serialize};

pub mod order_book;
pub mod orders;
pub mod trade;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserId(i64);

impl From<i64> for UserId {
    fn from(value: i64) -> Self {
        UserId(value)
    }
}

impl From<UserId> for i64 {
    fn from(value: UserId) -> Self {
        value.0
    }
}
#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
pub struct EpochTime(u128);

impl EpochTime {
    pub fn now() -> Result<Self, SystemTimeError> {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|duration| Self(duration.as_nanos()))
    }

    pub fn into_inner(self) -> u128 {
        self.0
    }
}
