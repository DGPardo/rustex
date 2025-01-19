use std::time::{SystemTime, SystemTimeError};

use serde::{Deserialize, Serialize};

pub mod order_book;
pub mod orders;
pub mod trade;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserId(u64);

impl From<u64> for UserId {
    fn from(value: u64) -> Self {
        UserId(value)
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
}
