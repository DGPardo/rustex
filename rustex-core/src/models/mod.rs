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
