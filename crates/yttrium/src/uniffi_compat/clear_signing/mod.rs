use crate::clear_signing::{
    format, DisplayItem, DisplayModel, EngineError, RawPreview,
};

const STAKE_WEIGHT_DESCRIPTOR: &str =
    include_str!("../../../tests/fixtures/stake_weight_descriptor.json");

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
}

impl From<EngineError> for EngineErrorFfi {
    fn from(value: EngineError) -> Self {
        match value {
            EngineError::DescriptorParse(err) => Self::DescriptorParse(err),
            EngineError::Calldata(err) => Self::Calldata(err),
            EngineError::Internal(err) => Self::Internal(err),
        }
    }
}

#[uniffi::export]
pub fn clear_signing_format(
    chain_id: u64,
    to: String,
    calldata: Vec<u8>,
) -> Result<DisplayModelFfi, EngineErrorFfi> {
    format(STAKE_WEIGHT_DESCRIPTOR, chain_id, &to, &calldata)
        .map(Into::into)
        .map_err(Into::into)
}
