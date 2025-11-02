use crate::clear_signing::{
    format as format_without_value, format_with_value, DisplayItem,
    DisplayModel, EngineError, RawPreview,
};

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
    #[error("resolver error: {0}")]
    Resolver(String),
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
            EngineError::Resolver(err) => Self::Resolver(err),
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
    clear_signing_format_with_value(chain_id, to, None, calldata_hex)
}

#[uniffi::export]
pub fn clear_signing_format_with_value(
    chain_id: u64,
    to: String,
    value_hex: Option<String>,
    calldata_hex: String,
) -> Result<DisplayModelFfi, EngineErrorFfi> {
    let calldata = decode_hex(&calldata_hex, "calldata")?;
    let value_bytes = match value_hex {
        Some(value_hex) => Some(decode_hex(&value_hex, "value")?),
        None => None,
    };

    let model = match value_bytes {
        Some(bytes) => {
            format_with_value(chain_id, &to, Some(bytes.as_slice()), &calldata)
        }
        None => format_without_value(chain_id, &to, &calldata),
    }
    .map_err(EngineErrorFfi::from)?;

    Ok(model.into())
}

fn decode_hex(input: &str, context: &str) -> Result<Vec<u8>, EngineErrorFfi> {
    let trimmed = input.trim();
    let without_prefix = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    hex::decode(without_prefix).map_err(|err| {
        EngineErrorFfi::Calldata(format!("invalid {context} hex: {err}"))
    })
}
