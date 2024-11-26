use crate::chain_abstraction::currency::Currency;
use alloy::primitives::{address, utils::Unit, Address};
use relay_rpc::domain::ProjectId;
use serde::{Deserialize, Serialize};

pub const FUNGIBLE_PRICE_ENDPOINT_PATH: &str = "/v1/fungible/price";

pub const NATIVE_TOKEN_ADDRESS: Address =
    address!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");

pub struct Caip10 {
    pub namespace: String,
    pub chain_id: String,
    pub token_address: Address,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceRequestBody {
    pub project_id: ProjectId,
    pub currency: Currency,
    pub addresses: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceResponseBody {
    pub fungibles: Vec<FungiblePriceItem>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FungiblePriceItem {
    pub name: String,
    pub symbol: String,
    pub icon_url: String,
    pub price: f64,
    #[serde(
        deserialize_with = "deserialize_unit",
        serialize_with = "serialize_unit"
    )]
    pub decimals: Unit,
}

fn deserialize_unit<'de, D>(deserializer: D) -> Result<Unit, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Unit::new(u8::deserialize(deserializer)?)
        .ok_or(serde::de::Error::custom("Unit must be less than 77"))
}

fn serialize_unit<S>(unit: &Unit, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_u8(unit.get())
}
