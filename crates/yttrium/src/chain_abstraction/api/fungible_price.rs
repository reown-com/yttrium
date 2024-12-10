use {
    crate::chain_abstraction::currency::Currency,
    alloy::primitives::{address, utils::Unit, Address},
    relay_rpc::domain::ProjectId,
    serde::{Deserialize, Serialize},
    std::collections::HashSet,
};

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
    pub addresses: HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceResponseBody {
    pub fungibles: Vec<FungiblePriceItem>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FungiblePriceItem {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub icon_url: String,
    pub price: f64,
    #[serde(
        deserialize_with = "crate::utils::deserialize_unit",
        serialize_with = "crate::utils::serialize_unit"
    )]
    pub decimals: Unit,
}
