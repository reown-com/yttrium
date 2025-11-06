use std::{collections::HashMap, sync::OnceLock};

use num_bigint::BigUint;
use thiserror::Error;
use time::{macros::format_description, OffsetDateTime};
use tiny_keccak::{Hasher, Keccak};

use super::{
    descriptor::{
        build_descriptor, decode_arguments, determine_token_key,
        resolve_effective_field, ArgumentValue, DecodedArguments, Descriptor,
        DescriptorError, DisplayField, DisplayFormat, EffectiveField,
        TokenLookupError, TokenLookupKey,
    },
    resolver::ResolvedCall,
    token_registry::TokenMeta,
};

const TETHER_USDT_DESCRIPTOR: &str =
    include_str!("assets/descriptors/erc20_usdt.json");
const UNISWAP_V3_ROUTER_DESCRIPTOR: &str =
    include_str!("assets/descriptors/uniswap_v3_router_v1.json");
const WETH9_DESCRIPTOR: &str = include_str!("assets/descriptors/weth9.json");

const ADDRESS_BOOK_DESCRIPTORS: &[&str] =
    &[TETHER_USDT_DESCRIPTOR, UNISWAP_V3_ROUTER_DESCRIPTOR, WETH9_DESCRIPTOR];

static GLOBAL_ADDRESS_BOOK: OnceLock<HashMap<String, String>> = OnceLock::new();

/// Minimal display item for the clear signing preview.
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayItem {
    pub label: String,
    pub value: String,
}

/// Display model produced by the clear signing engine.
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayModel {
    pub intent: String,
    pub items: Vec<DisplayItem>,
    pub warnings: Vec<String>,
    pub raw: Option<RawPreview>,
}

/// Raw fallback preview details.
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawPreview {
    pub selector: String,
    pub args: Vec<String>,
}

/// Errors returned by the clear signing engine.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("descriptor parse error: {0}")]
    DescriptorParse(String),
    #[error("calldata decode error: {0}")]
    Calldata(String),
    #[error("resolver error: {0}")]
    Resolver(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error("token registry error: {0}")]
    TokenRegistry(String),
}

impl From<DescriptorError> for EngineError {
    fn from(err: DescriptorError) -> Self {
        match err {
            DescriptorError::Parse(message) => {
                EngineError::DescriptorParse(message)
            }
            DescriptorError::Calldata(message) => {
                EngineError::Calldata(message)
            }
        }
    }
}

/// Decodes calldata using a previously resolved descriptor bundle and returns
/// a human-readable preview.
pub fn format_with_resolved_call(
    resolved: ResolvedCall<'_>,
    chain_id: u64,
    to: &str,
    value: Option<&[u8]>,
    calldata: &[u8],
) -> Result<DisplayModel, EngineError> {
    eprintln!(
        "[engine] format_with_resolved chain_id={} to={} calldata_len={}",
        chain_id,
        to,
        calldata.len()
    );
    let token_metadata = resolved.token_metadata;
    let descriptor = build_descriptor(&resolved.descriptor)?;

    let mut warnings = Vec::new();

    if !descriptor.context.contract.is_bound_to(chain_id, to) {
        warnings.push(format!(
            "Descriptor deployment mismatch for chain {chain_id} and address {to}"
        ));
    }

    let selector = extract_selector(calldata)?;
    let selector_hex = format_selector_hex(&selector);

    let functions = descriptor.context.contract.function_descriptors()?;
    let display_formats = descriptor.display.format_map();
    let local_address_book = descriptor_address_book(&descriptor);

    let Some(function) =
        functions.iter().find(|func| func.selector == selector)
    else {
        warnings.push(format!("No ABI match for selector {selector_hex}"));
        eprintln!("[engine] no ABI match for selector {}", selector_hex);
        return Ok(DisplayModel {
            intent: "Unknown transaction".to_string(),
            items: Vec::new(),
            warnings,
            raw: Some(raw_preview_from_calldata(&selector, calldata)),
        });
    };

    let decoded = decode_arguments(function, calldata)?;
    eprintln!("[engine] decoded arguments count {}", decoded.ordered().len());
    let decoded = decoded.with_value(value)?;
    eprintln!("[engine] decoded with value count {}", decoded.ordered().len());

    if let Some(format_def) = display_formats.get(&function.typed_signature) {
        let (items, mut format_warnings) = apply_display_format(
            format_def,
            &decoded,
            &descriptor.metadata,
            chain_id,
            to,
            &local_address_book,
            descriptor.display.definitions(),
            &token_metadata,
        )?;
        warnings.append(&mut format_warnings);
        Ok(DisplayModel {
            intent: format_def.intent.clone(),
            items,
            warnings,
            raw: None,
        })
    } else {
        warnings.push(format!(
            "No display format defined for signature {}",
            function.typed_signature
        ));
        let items = decoded
            .ordered()
            .iter()
            .map(|arg| DisplayItem {
                label: arg.display_label(),
                value: arg.value.default_string(),
            })
            .collect();
        Ok(DisplayModel {
            intent: "Transaction".to_string(),
            items,
            warnings,
            raw: Some(RawPreview {
                selector: selector_hex,
                args: decoded
                    .ordered()
                    .iter()
                    .map(|arg| arg.raw_word_hex())
                    .collect(),
            }),
        })
    }
}

/// Convenience helper to call the engine directly with raw JSON assets.
#[allow(dead_code)]
fn apply_display_format(
    format: &DisplayFormat,
    decoded: &DecodedArguments,
    metadata: &serde_json::Value,
    chain_id: u64,
    contract_address: &str,
    address_book: &HashMap<String, String>,
    definitions: &HashMap<String, DisplayField>,
    token_metadata: &HashMap<TokenLookupKey, TokenMeta>,
) -> Result<(Vec<DisplayItem>, Vec<String>), EngineError> {
    let mut items = Vec::new();
    let mut warnings = Vec::new();

    for required in &format.required {
        if decoded.get(required).is_none() {
            warnings.push(format!("Missing required argument '{required}'"));
        }
    }

    for field in &format.fields {
        let Some(effective) =
            resolve_effective_field(field, definitions, &mut warnings)
        else {
            continue;
        };

        if let Some(value) = decoded.get(&effective.path) {
            let rendered = render_field(
                &effective,
                value,
                decoded,
                metadata,
                chain_id,
                contract_address,
                address_book,
                token_metadata,
            )?;
            items.push(DisplayItem {
                label: effective.label.clone(),
                value: rendered,
            });
        } else {
            warnings.push(format!(
                "No value found for field path '{}'",
                effective.path
            ));
        }
    }

    Ok((items, warnings))
}

fn render_field(
    field: &EffectiveField,
    value: &ArgumentValue,
    decoded: &DecodedArguments,
    metadata: &serde_json::Value,
    chain_id: u64,
    contract_address: &str,
    address_book: &HashMap<String, String>,
    token_metadata: &HashMap<TokenLookupKey, TokenMeta>,
) -> Result<String, EngineError> {
    match field.format.as_deref() {
        Some("date") => Ok(format_date(value)),
        Some("tokenAmount") => format_token_amount(
            field,
            value,
            decoded,
            metadata,
            chain_id,
            contract_address,
            token_metadata,
        ),
        Some("amount") => {
            Ok(format_native_amount(value, chain_id, token_metadata))
        }
        Some("address") | Some("addressName") => {
            Ok(format_address(value, address_book))
        }
        Some("enum") => format_enum(field, value, metadata),
        Some("number") => Ok(format_number(value)),
        _ => Ok(value.default_string()),
    }
}

fn format_date(value: &ArgumentValue) -> String {
    let ArgumentValue::Uint(amount) = value else {
        return value.default_string();
    };

    let Ok(seconds) = u64::try_from(amount.clone()) else {
        return value.default_string();
    };
    let Ok(timestamp) = i64::try_from(seconds) else {
        return value.default_string();
    };
    let Ok(datetime) = OffsetDateTime::from_unix_timestamp(timestamp) else {
        return value.default_string();
    };

    let format = format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second] UTC"
    );
    datetime
        .to_offset(time::UtcOffset::UTC)
        .format(&format)
        .unwrap_or_else(|_| value.default_string())
}

fn format_token_amount(
    field: &EffectiveField,
    value: &ArgumentValue,
    decoded: &DecodedArguments,
    metadata: &serde_json::Value,
    chain_id: u64,
    contract_address: &str,
    token_metadata: &HashMap<TokenLookupKey, TokenMeta>,
) -> Result<String, EngineError> {
    let ArgumentValue::Uint(amount) = value else {
        return Ok(value.default_string());
    };

    if let Some(message) = token_amount_message(field, amount, metadata) {
        return Ok(message);
    }

    let token_meta = lookup_token_meta(
        field,
        decoded,
        chain_id,
        contract_address,
        token_metadata,
    )?;
    let formatted_amount =
        format_amount_with_decimals(amount, token_meta.decimals);
    Ok(format!("{} {}", formatted_amount, token_meta.symbol))
}

fn format_address(
    value: &ArgumentValue,
    local_address_book: &HashMap<String, String>,
) -> String {
    let Some(bytes) = value.as_address() else {
        return value.default_string();
    };

    let checksum = to_checksum_address(bytes);
    let normalized = checksum.to_ascii_lowercase();

    if let Some(label) = local_address_book.get(&normalized) {
        return label.clone();
    }

    if let Some(label) = global_address_book().get(&normalized) {
        return label.clone();
    }

    checksum
}

fn format_number(value: &ArgumentValue) -> String {
    let Some(number) = value.as_uint() else {
        return value.default_string();
    };
    number.to_string()
}

fn format_native_amount(
    value: &ArgumentValue,
    chain_id: u64,
    token_metadata: &HashMap<TokenLookupKey, TokenMeta>,
) -> String {
    let ArgumentValue::Uint(amount) = value else {
        return value.default_string();
    };

    if let Some(slip44) = native_slip44_code(chain_id) {
        let caip19 = format!("eip155:{}/slip44:{}", chain_id, slip44);
        if let Some(meta) =
            token_metadata.get(&TokenLookupKey::Caip19(caip19))
        {
            let formatted = format_amount_with_decimals(amount, meta.decimals);
            return format!("{} {}", formatted, meta.symbol);
        }
    }

    let formatted = format_amount_with_decimals(amount, 18);
    format!("{} ETH", formatted)
}

fn native_slip44_code(chain_id: u64) -> Option<u32> {
    match chain_id {
        1 | 10 | 42161 | 8453 => Some(60),
        _ => None,
    }
}

fn map_token_lookup_error(err: TokenLookupError) -> EngineError {
    EngineError::TokenRegistry(err.to_string())
}

fn format_enum(
    field: &EffectiveField,
    value: &ArgumentValue,
    metadata: &serde_json::Value,
) -> Result<String, EngineError> {
    let ArgumentValue::Uint(amount) = value else {
        return Ok(value.default_string());
    };

    let Some(reference) = field.params.get("$ref").and_then(|v| v.as_str())
    else {
        return Ok(value.default_string());
    };

    let Some(enum_map) = resolve_metadata_value(metadata, reference) else {
        return Ok(value.default_string());
    };

    if let Some(mapping) = enum_map.as_object() {
        if let Some(label) = mapping.get(&amount.to_string()) {
            if let Some(text) = label.as_str() {
                return Ok(text.to_string());
            }
        }
    }

    Ok(amount.to_string())
}

fn token_amount_message(
    field: &EffectiveField,
    amount: &BigUint,
    metadata: &serde_json::Value,
) -> Option<String> {
    let threshold_spec =
        field.params.get("threshold").and_then(|value| value.as_str())?;
    let message =
        field.params.get("message").and_then(|value| value.as_str())?;

    let threshold = if threshold_spec.starts_with("$.") {
        resolve_metadata_biguint(metadata, threshold_spec)?
    } else {
        parse_biguint(threshold_spec)?
    };
    if amount >= &threshold {
        Some(message.to_string())
    } else {
        None
    }
}

fn lookup_token_meta(
    field: &EffectiveField,
    decoded: &DecodedArguments,
    chain_id: u64,
    contract_address: &str,
    token_metadata: &HashMap<TokenLookupKey, TokenMeta>,
) -> Result<TokenMeta, EngineError> {
    let key = determine_token_key(field, decoded, chain_id, contract_address)
        .map_err(map_token_lookup_error)?;
    token_metadata.get(&key).cloned().ok_or_else(|| {
        EngineError::TokenRegistry(format!(
            "token registry missing entry for {:?}",
            key
        ))
    })
}

pub(crate) fn format_amount_with_decimals(
    amount: &BigUint,
    decimals: u8,
) -> String {
    if decimals == 0 {
        return add_thousand_separators(&amount.to_string());
    }

    let factor = BigUint::from(10u32).pow(decimals as u32);
    let integer = amount / &factor;
    let remainder = amount % &factor;

    let integer_part = add_thousand_separators(&integer.to_string());
    if remainder == BigUint::from(0u32) {
        return integer_part;
    }

    let mut fractional = remainder.to_string();
    let width = decimals as usize;
    if fractional.len() < width {
        let mut padded = String::with_capacity(width);
        for _ in 0..(width - fractional.len()) {
            padded.push('0');
        }
        padded.push_str(&fractional);
        fractional = padded;
    }

    while fractional.ends_with('0') {
        fractional.pop();
    }

    if fractional.is_empty() {
        integer_part
    } else {
        format!("{}.{}", integer_part, fractional)
    }
}

pub(crate) fn add_thousand_separators(value: &str) -> String {
    let mut reversed = String::with_capacity(value.len());
    for (index, ch) in value.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            reversed.push(',');
        }
        reversed.push(ch);
    }
    reversed.chars().rev().collect()
}

fn resolve_metadata_biguint(
    metadata: &serde_json::Value,
    pointer: &str,
) -> Option<BigUint> {
    let value = resolve_metadata_value(metadata, pointer)?;
    if let Some(text) = value.as_str() {
        parse_biguint(text)
    } else if let Some(number) = value.as_u64() {
        Some(BigUint::from(number))
    } else {
        None
    }
}

pub(crate) fn resolve_metadata_value<'a>(
    metadata: &'a serde_json::Value,
    pointer: &str,
) -> Option<&'a serde_json::Value> {
    const PREFIX: &str = "$.metadata.";
    let rest = pointer.strip_prefix(PREFIX)?;
    let mut current = metadata;
    for segment in rest.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

pub(crate) fn parse_biguint(text: &str) -> Option<BigUint> {
    if let Some(hex) = text.strip_prefix("0x") {
        BigUint::parse_bytes(hex.as_bytes(), 16)
    } else {
        BigUint::parse_bytes(text.as_bytes(), 10)
    }
}

fn raw_preview_from_calldata(
    selector: &[u8; 4],
    calldata: &[u8],
) -> RawPreview {
    let args = if calldata.len() > 4 {
        calldata[4..]
            .chunks(32)
            .map(|chunk| format!("0x{}", hex::encode(chunk)))
            .collect()
    } else {
        Vec::new()
    };

    RawPreview { selector: format_selector_hex(selector), args }
}

fn extract_selector(calldata: &[u8]) -> Result<[u8; 4], EngineError> {
    if calldata.len() < 4 {
        return Err(EngineError::Calldata(
            "calldata must be at least 4 bytes".to_string(),
        ));
    }
    let mut selector = [0u8; 4];
    selector.copy_from_slice(&calldata[0..4]);
    Ok(selector)
}

fn format_selector_hex(selector: &[u8; 4]) -> String {
    format!("0x{}", hex::encode(selector))
}

fn global_address_book() -> &'static HashMap<String, String> {
    GLOBAL_ADDRESS_BOOK.get_or_init(|| {
        let mut map = HashMap::new();
        for descriptor_json in ADDRESS_BOOK_DESCRIPTORS {
            if let Ok(parsed) =
                serde_json::from_str::<Descriptor>(descriptor_json)
            {
                let local_book = descriptor_address_book(&parsed);
                for (address, label) in local_book {
                    map.entry(address).or_insert(label);
                }
            }
        }
        map
    })
}

fn descriptor_address_book(descriptor: &Descriptor) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(label) = descriptor_friendly_label(descriptor) {
        for deployment in &descriptor.context.contract.deployments {
            map.insert(normalize_address(&deployment.address), label.clone());
        }
    }
    map
}

fn descriptor_friendly_label(descriptor: &Descriptor) -> Option<String> {
    let metadata = &descriptor.metadata;

    if let Some(token) = metadata.get("token") {
        let name = token.get("name").and_then(|value| value.as_str());
        let symbol = token.get("symbol").and_then(|value| value.as_str());
        match (name, symbol) {
            (Some(name), Some(symbol)) => {
                if name.eq_ignore_ascii_case(symbol) {
                    return Some(name.to_string());
                } else {
                    return Some(format!("{name} ({symbol})"));
                }
            }
            (Some(name), None) => return Some(name.to_string()),
            (None, Some(symbol)) => return Some(symbol.to_string()),
            (None, None) => {}
        }
    }

    if let Some(info) = metadata.get("info") {
        if let Some(name) =
            info.get("legalName").and_then(|value| value.as_str())
        {
            return Some(name.to_string());
        }
        if let Some(name) = info.get("name").and_then(|value| value.as_str()) {
            return Some(name.to_string());
        }
    }

    if let Some(owner) = metadata.get("owner").and_then(|value| value.as_str())
    {
        return Some(owner.to_string());
    }

    descriptor.context.id.clone()
}

fn normalize_address(address: &str) -> String {
    address.trim().to_ascii_lowercase()
}

pub(crate) fn to_checksum_address(bytes: &[u8; 20]) -> String {
    let lower = hex::encode(bytes);
    let mut hasher = Keccak::v256();
    hasher.update(lower.as_bytes());
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);

    let mut result = String::with_capacity(42);
    result.push_str("0x");
    for (index, ch) in lower.chars().enumerate() {
        if ch.is_ascii_hexdigit() && ch.is_ascii_alphabetic() {
            let hash_byte = hash[index / 2];
            let nibble = if index % 2 == 0 {
                (hash_byte >> 4) & 0x0f
            } else {
                hash_byte & 0x0f
            };
            if nibble >= 8 {
                result.push(ch.to_ascii_uppercase());
                continue;
            }
        }
        result.push(ch);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
}
