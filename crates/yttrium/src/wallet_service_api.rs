use {
    alloy::primitives::{Address, U64, U256},
    serde::{Deserialize, Deserializer, Serialize, Serializer, de},
    std::collections::HashMap,
};

pub const WALLET_GET_ASSETS: &str = "wallet_getAssets";

// https://github.com/ethereum/ERCs/pull/709/files#diff-be675f3ce6b6aa5616dd1bccf5e50f44ad65775afb967a47aaffb8f5eb51b849R35
#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi_macros::Record))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetsParams {
    pub account: Address,
    #[serde(flatten)]
    pub filters: GetAssetsFilters,
}

#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi_macros::Record))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetsFilters {
    #[serde(default)]
    pub asset_filter: Option<AssetFilter>,
    #[serde(default)]
    pub asset_type_filter: Option<AssetTypeFilter>,
    #[serde(default)]
    pub chain_filter: Option<ChainFilter>,
}

pub type AssetFilter = HashMap<Eip155ChainId, Vec<AddressOrNative>>;
pub type AssetTypeFilter = Vec<AssetType>;
pub type ChainFilter = Vec<Eip155ChainId>;

#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi_macros::Enum))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
pub enum AssetType {
    Native,
    Erc20,
    Erc721,
}

pub type Eip155ChainId = U64;
pub type GetAssetsResult = HashMap<Eip155ChainId, Vec<Asset>>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
pub enum Asset {
    Native {
        #[serde(flatten)]
        data: AssetData<NativeMetadata>,
    },
    Erc20 {
        #[serde(flatten)]
        data: AssetData<Erc20Metadata>,
    },
    Erc721 {
        #[serde(flatten)]
        data: AssetData<Erc721Metadata>,
    },
}

impl Asset {
    pub fn balance(&self) -> U256 {
        match self {
            Self::Native { data } => data.balance,
            Self::Erc20 { data } => data.balance,
            Self::Erc721 { data } => data.balance,
        }
    }

    pub fn asset_type(&self) -> AssetType {
        match self {
            Self::Native { .. } => AssetType::Native,
            Self::Erc20 { .. } => AssetType::Erc20,
            Self::Erc721 { .. } => AssetType::Erc721,
        }
    }

    pub fn as_erc20(&self) -> Option<&AssetData<Erc20Metadata>> {
        match self {
            Self::Erc20 { data } => Some(data),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi_macros::Enum))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub enum AddressOrNative {
    // https://github.com/mozilla/uniffi-rs/issues/2402
    AddressVariant(Address),
    Native,
}

impl AddressOrNative {
    pub fn as_address(&self) -> Option<&Address> {
        match self {
            Self::AddressVariant(address) => Some(address),
            Self::Native => None,
        }
    }
}

impl Serialize for AddressOrNative {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            AddressOrNative::Native => serializer.serialize_str("native"),
            AddressOrNative::AddressVariant(address) => {
                address.serialize(serializer)
            }
        }
    }
}

impl<'de> Deserialize<'de> for AddressOrNative {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the input as a string.
        let s = String::deserialize(deserializer)?;

        if s == "native" {
            Ok(AddressOrNative::Native)
        } else {
            s.parse::<Address>()
                .map_err(de::Error::custom)
                .map(AddressOrNative::AddressVariant)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct AssetData<M> {
    pub address: AddressOrNative,
    pub balance: U256,
    pub metadata: M,
}

#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi_macros::Record))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct NativeMetadata {
    // Also non-standard data (for now?)
    pub name: String,
    pub symbol: String,
    pub decimals: u8,

    // Non-standard data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    pub price: f64,
    pub icon_url: String,
}

#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi_macros::Record))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct Erc20Metadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,

    // Non-standard data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    pub price: f64,
    pub icon_url: String,
}

#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi_macros::Record))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct Erc721Metadata {
    pub name: String,
    pub symbol: String,
}
