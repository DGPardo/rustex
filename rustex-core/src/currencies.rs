// Hopefully at some point this can be retrieved dynamically from a database
// such that adding a new currency does not require a code change

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Currencies {
    BTC,
    USD,
    GBP,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum ExchangeMarkets {
    BTC_USD,
    BTC_GBP,
    GBP_USD,
}
