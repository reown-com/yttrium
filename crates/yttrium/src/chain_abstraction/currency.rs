use {
    alloy::primitives::utils::Unit,
    serde::{Deserialize, Serialize},
};

// TODO get Blockchain API to use these types?

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi_macros::Enum))]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[serde(rename_all = "lowercase")]
pub enum Currency {
    Usd,
    Eur,
    Gbp,
    Aud,
    Cad,
    Inr,
    Jpy,
    Btc,
    Eth,
}

impl Currency {
    pub fn symbol(&self) -> &str {
        match self {
            Currency::Usd => "USD",
            Currency::Eur => "EUR",
            Currency::Gbp => "GBP",
            Currency::Aud => "AUD",
            Currency::Cad => "CAD",
            Currency::Inr => "INR",
            Currency::Jpy => "JPY",
            Currency::Btc => "BTC",
            Currency::Eth => "ETH",
        }
    }

    pub fn unit(&self) -> Unit {
        match self {
            // TODO not sure if all of these are right
            Currency::Usd => Unit::new(2).unwrap(),
            Currency::Eur => Unit::new(2).unwrap(),
            Currency::Gbp => Unit::new(2).unwrap(),
            Currency::Aud => Unit::new(2).unwrap(),
            Currency::Cad => Unit::new(2).unwrap(),
            Currency::Inr => Unit::new(2).unwrap(),
            Currency::Jpy => Unit::new(0).unwrap(),
            Currency::Btc => Unit::new(8).unwrap(),
            Currency::Eth => Unit::new(18).unwrap(),
        }
    }
}
