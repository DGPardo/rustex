// Hopefully at some point this can be retrieved dynamically from a database
// such that adding a new currency does not require a code change

use std::str::FromStr;

use rustex_errors::RustexError;
use serde::{Deserialize, Serialize};

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Currencies {
    BTC,
    USD,
    GBP,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExchangeMarkets {
    BTC_USD,
    BTC_GBP,
    BTC_EUR,
}

impl FromStr for ExchangeMarkets {
    type Err = RustexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "BTC_USD" => Ok(ExchangeMarkets::BTC_USD),
            "BTC_GBP" => Ok(ExchangeMarkets::BTC_GBP),
            "BTC_EUR" => Ok(ExchangeMarkets::BTC_EUR),
            _ => Err(RustexError::UserFacingError(format!(
                "{s} is not a valid Exchange Marker"
            ))),
        }
    }
}

impl ExchangeMarkets {
    pub fn from_env() -> Result<Self, RustexError> {
        std::env::var("EXCHANGE_MARKET")
            .map(|env_var| ExchangeMarkets::from_str(&env_var))
            .map_err(|_| {
                RustexError::UserFacingError(
                    "EXCHANGE_MARKET environment variable is undefined".into(),
                )
            })?
    }
}
