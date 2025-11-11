use crate::clear_signing::{
    format as format_without_value, format_typed_data, format_with_value,
    DisplayModel, Eip712Error, EngineError, TypedData,
};

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

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, uniffi::Enum)]
pub enum TypedEngineErrorFfi {
    #[error("typed data parse error: {0}")]
    TypedData(String),
    #[error("resolver error: {0}")]
    Resolver(String),
    #[error("descriptor error: {0}")]
    Descriptor(String),
    #[error("token registry error: {0}")]
    TokenRegistry(String),
}

impl From<Eip712Error> for TypedEngineErrorFfi {
    fn from(value: Eip712Error) -> Self {
        match value {
            Eip712Error::TypedData(err) => Self::TypedData(err),
            Eip712Error::Resolver(err) => Self::Resolver(err),
            Eip712Error::DescriptorParse(err) => Self::Descriptor(err),
            Eip712Error::TokenRegistry(err) => Self::TokenRegistry(err),
        }
    }
}

#[uniffi::export]
pub fn clear_signing_format(
    chain_id: u64,
    to: String,
    calldata_hex: String,
) -> Result<DisplayModel, EngineErrorFfi> {
    clear_signing_format_with_value(chain_id, to, None, calldata_hex)
}

#[uniffi::export]
pub fn clear_signing_format_with_value(
    chain_id: u64,
    to: String,
    value_hex: Option<String>,
    calldata_hex: String,
) -> Result<DisplayModel, EngineErrorFfi> {
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

    Ok(model)
}

fn decode_hex(input: &str, context: &str) -> Result<Vec<u8>, EngineErrorFfi> {
    let trimmed = input.trim();
    let without_prefix = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    hex::decode(without_prefix).map_err(|err| {
        EngineErrorFfi::Calldata(format!("invalid {context} hex: {err}"))
    })
}

#[uniffi::export]
pub fn clear_signing_format_typed(
    typed_data_json: String,
) -> Result<DisplayModel, TypedEngineErrorFfi> {
    let typed: TypedData = serde_json::from_str(&typed_data_json)
        .map_err(|err| TypedEngineErrorFfi::TypedData(err.to_string()))?;
    format_typed_data(&typed).map_err(Into::into)
}
