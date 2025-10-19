use crate::{
    clear_signing::{
        format, DisplayItem, DisplayModel, EngineError, RawPreview,
    },
    descriptors::aave::{AAVE_LPV2, AAVE_LPV3, AAVE_WETH_GATEWAY_V3},
};

const STAKE_WEIGHT_DESCRIPTOR: &str =
    include_str!("../../../tests/fixtures/stake_weight_descriptor.json");
const TETHER_USDT_DESCRIPTOR: &str =
    include_str!("../../../../../vendor/registry/tether/calldata-usdt.json");
const AAVE_LPV2_DESCRIPTOR: &str = AAVE_LPV2;
const AAVE_LPV3_DESCRIPTOR: &str = AAVE_LPV3;
const AAVE_WETH_GATEWAY_V3_DESCRIPTOR: &str = AAVE_WETH_GATEWAY_V3;

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct DisplayItemFfi {
    pub label: String,
    pub value: String,
}

impl From<DisplayItem> for DisplayItemFfi {
    fn from(value: DisplayItem) -> Self {
        Self { label: value.label, value: value.value }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct RawPreviewFfi {
    pub selector: String,
    pub args: Vec<String>,
}

impl From<RawPreview> for RawPreviewFfi {
    fn from(value: RawPreview) -> Self {
        Self { selector: value.selector, args: value.args }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct DisplayModelFfi {
    pub intent: String,
    pub items: Vec<DisplayItemFfi>,
    pub warnings: Vec<String>,
    pub raw: Option<RawPreviewFfi>,
}

impl From<DisplayModel> for DisplayModelFfi {
    fn from(value: DisplayModel) -> Self {
        Self {
            intent: value.intent,
            items: value.items.into_iter().map(Into::into).collect(),
            warnings: value.warnings,
            raw: value.raw.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, uniffi::Enum)]
pub enum EngineErrorFfi {
    #[error("descriptor parse error: {0}")]
    DescriptorParse(String),
    #[error("calldata error: {0}")]
    Calldata(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error("token registry error: {0}")]
    TokenRegistry(String),
}

impl From<EngineError> for EngineErrorFfi {
    fn from(value: EngineError) -> Self {
        match value {
            EngineError::DescriptorParse(err) => Self::DescriptorParse(err),
            EngineError::Calldata(err) => Self::Calldata(err),
            EngineError::Internal(err) => Self::Internal(err),
            EngineError::TokenRegistry(err) => Self::TokenRegistry(err),
        }
    }
}

#[uniffi::export]
pub fn clear_signing_format(
    chain_id: u64,
    to: String,
    calldata_hex: String,
) -> Result<DisplayModelFfi, EngineErrorFfi> {
    println!(
        "[clear_signing_ffi] chain_id={} to={} calldata_hex_len={}",
        chain_id,
        to,
        calldata_hex.len()
    );
    let calldata = decode_calldata(&calldata_hex)?;
    let descriptor_json = select_descriptor(chain_id, &to);
    println!(
        "[clear_signing_ffi] descriptor json length={} preview={}",
        descriptor_json.len(),
        descriptor_json.chars().take(120).collect::<String>()
    );
    format(descriptor_json, chain_id, &to, &calldata)
        .map(Into::into)
        .map_err(Into::into)
}

fn select_descriptor(_chain_id: u64, to: &str) -> &'static str {
    let normalized_to = to.trim().to_ascii_lowercase();
    address_descriptor(normalized_to.as_str())
        .unwrap_or(STAKE_WEIGHT_DESCRIPTOR)
}

fn decode_calldata(calldata_hex: &str) -> Result<Vec<u8>, EngineErrorFfi> {
    let trimmed = calldata_hex.trim();
    let without_prefix = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    let bytes = hex::decode(without_prefix).map_err(|err| {
        EngineErrorFfi::Calldata(format!("invalid hex calldata: {}", err))
    })?;
    Ok(bytes)
}

fn address_descriptor(address: &str) -> Option<&'static str> {
    const TETHER_ADDRESSES: &[&str] = &[
        "0xdac17f958d2ee523a2206206994597c13d831ec7",
        "0x94b008aa00579c1307b0ef2c499ad98a8ce58e58",
        "0xc2132d05d31c914a87c6611c10748aeb04b58e8f",
    ];

    const AAVE_LPV2_ADDRESSES: &[&str] = &[
        "0x7d2768de32b0b80b7a3454c06bdac94a69ddc7a9",
        "0x8dff5e27ea6b7ac08ebfdf9eb090f32ee9a30fcf",
        "0x4f01aed16d97e3ab5ab2b501154dc9bb0f1a5a2c",
    ];

    const AAVE_LPV3_ADDRESSES: &[&str] = &[
        "0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2",
        "0xa238dd80c259a72e81d7e4664a9801593f98d1c5",
        "0x3e59a31363e2ad014dcbc521c4a0d5757d9f3402",
        "0xc47b8c00b0f69a36fa203ffeac0334874574a8ac",
        "0x90df02551bb792286e8d4f13e0e357b4bf1d6a57",
        "0x5362dbb1e601abf3a4c14c22ffeda64042e5eaa3",
        "0xb50201558b00496a145fe76f7424749556e326d8",
        "0x11fcfe756c05ad438e312a7fd934381537d3cffe",
        "0x78e30497a3c7527d953c6b1e3541b021a98ac43c",
        "0x794a61358d6845594f94dc1db02a252b5b4814ad",
        "0xdd3d7a7d03d9fd9ef45f3e587287922ef65ca38b",
        "0x925a2a7214ed92428b5b1b090f80b25700095e12",
    ];

    const AAVE_WETH_GATEWAY_V3_ADDRESSES: &[&str] = &[
        "0xd01607c3c5ecaba394d8be377a08590149325722",
        "0x5f2508cae9923b02316254026cd43d7902866725",
        "0x721b9abab6511b46b9ee83a1aba23bdacb004149",
        "0xbc302053db3aa514a3c86b9221082f162b91ad63",
        "0x061d8e131f26512348ee5fa42e2df1ba9d6505e9",
        "0xae2b00d676130bdf22582781bbba8f4f21e8b0ff",
        "0x6376d4df995f32f308f2d5049a7a320943023232",
        "0xa0d9c1e9e48ca30c8d8c3b5d69ff5dc1f6dffc24",
        "0x54bdcc37c4143f944a3ee51c892a6cbdf305e7a0",
        "0x5283beced7adf6d003225c13896e536f2d4264ff",
        "0x2825ce5921538d17cc15ae00a8b24ff759c6cdae",
        "0x31a239f3e39c5d8ba6b201ba81ed584492ae960f",
        "0xe79ca44408dae5a57ea2a9594532f1e84d2edaa4",
    ];

    if TETHER_ADDRESSES.contains(&address) {
        Some(TETHER_USDT_DESCRIPTOR)
    } else if AAVE_LPV3_ADDRESSES.contains(&address) {
        Some(AAVE_LPV3_DESCRIPTOR)
    } else if AAVE_LPV2_ADDRESSES.contains(&address) {
        Some(AAVE_LPV2_DESCRIPTOR)
    } else if AAVE_WETH_GATEWAY_V3_ADDRESSES.contains(&address) {
        Some(AAVE_WETH_GATEWAY_V3_DESCRIPTOR)
    } else {
        None
    }
}
