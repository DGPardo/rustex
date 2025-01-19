use std::time::{SystemTime, SystemTimeError};

use serde::{Deserialize, Serialize};

pub mod order_book;
pub mod orders;
pub mod trade;

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
pub struct EpochTime(u128);

impl EpochTime {
    pub fn now() -> Result<Self, SystemTimeError> {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|duration| Self(duration.as_nanos()))
    }
}
