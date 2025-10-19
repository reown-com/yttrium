use crate::clear_signing::{
    format, DisplayItem, DisplayModel, EngineError, RawPreview,
};

const STAKE_WEIGHT_DESCRIPTOR: &str =
    include_str!("../../../tests/fixtures/stake_weight_descriptor.json");
const TETHER_USDT_DESCRIPTOR: &str =
    include_str!("../../../../../vendor/registry/tether/calldata-usdt.json");

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

fn select_descriptor(chain_id: u64, to: &str) -> &'static str {
    let normalized_to = to.trim().to_ascii_lowercase();
    let descriptor = match (chain_id, normalized_to.as_str()) {
        (1, "0xdac17f958d2ee523a2206206994597c13d831ec7")
        | (10, "0x94b008aa00579c1307b0ef2c499ad98a8ce58e58")
        | (137, "0xc2132d05d31c914a87c6611c10748aeb04b58e8f") => {
            TETHER_USDT_DESCRIPTOR
        }
        _ => STAKE_WEIGHT_DESCRIPTOR,
    };
    descriptor
}

fn decode_calldata(calldata_hex: &str) -> Result<Vec<u8>, EngineErrorFfi> {
    let trimmed = calldata_hex.trim();
    let without_prefix = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    let bytes = hex::decode(without_prefix).map_err(|err| {
        EngineErrorFfi::Calldata(format!("invalid hex calldata: {}", err))
    })?;
    Ok(bytes)
}
