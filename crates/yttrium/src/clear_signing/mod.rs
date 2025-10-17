use std::collections::HashMap;

use num_bigint::BigUint;
use serde::Deserialize;
use thiserror::Error;
use time::{macros::format_description, OffsetDateTime};
use tiny_keccak::{Hasher, Keccak};

mod token_registry;
pub use token_registry::{lookup_token, TokenMeta};

use crate::descriptors::aave::{
    AAVE_LPV2_ABI, AAVE_LPV3_ABI, AAVE_WETH_GATEWAY_V3_ABI,
};

/// Minimal display item for the clear signing preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayItem {
    pub label: String,
    pub value: String,
}

/// Display model produced by the clear signing engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayModel {
    pub intent: String,
    pub items: Vec<DisplayItem>,
    pub warnings: Vec<String>,
    pub raw: Option<RawPreview>,
}

/// Raw fallback preview details.
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
    #[error("internal error: {0}")]
    Internal(String),
    #[error("token registry error: {0}")]
    TokenRegistry(String),
}

/// Parses the ERC-7730 descriptor, binds to (chain_id,to), decodes calldata via ABI, selects a display
/// format, and returns a preview.
pub fn format(
    descriptor_json: &str,
    chain_id: u64,
    to: &str,
    calldata: &[u8],
) -> Result<DisplayModel, EngineError> {
    println!(
        "[clear_signing] start chain_id={} to={} calldata_len={} calldata=0x{}",
        chain_id,
        to,
        calldata.len(),
        hex::encode(calldata)
    );
    println!(
        "[clear_signing] descriptor json length={} preview={}",
        descriptor_json.len(),
        &descriptor_json.chars().take(120).collect::<String>()
    );
    let descriptor: Descriptor = serde_json::from_str(descriptor_json)
        .map_err(|err| EngineError::DescriptorParse(err.to_string()))?;
    println!("[clear_signing] descriptor parsed");

    let mut warnings = Vec::new();

    if !descriptor.context.contract.is_bound_to(chain_id, to) {
        warnings.push(format!(
            "Descriptor deployment mismatch for chain {chain_id} and address {to}"));
        println!(
            "[clear_signing] deployment mismatch chain={} to={}",
            chain_id, to
        );
    }

    let selector = extract_selector(calldata)?;
    let selector_hex = format_selector_hex(&selector);
    println!("[clear_signing] selector extracted {}", selector_hex);

    let functions = descriptor.context.contract.function_descriptors()?;
    println!("[clear_signing] functions available {}", functions.len());
    for func in &functions {
        println!(
            "[clear_signing] function {} selector=0x{}",
            func.typed_signature,
            hex::encode(func.selector)
        );
    }
    let display_formats = descriptor.display.format_map();
    println!(
        "[clear_signing] display formats available {}",
        display_formats.len()
    );

    let maybe_function =
        functions.iter().find(|func| func.selector == selector);

    if maybe_function.is_none() {
        println!("[clear_signing] no ABI match for selector {}", selector_hex);
    }

    let Some(function) = maybe_function else {
        warnings.push(format!("No ABI match for selector {selector_hex}"));
        return Ok(DisplayModel {
            intent: "Unknown transaction".to_string(),
            items: Vec::new(),
            warnings,
            raw: Some(raw_preview_from_calldata(&selector, calldata)),
        });
    };

    println!("[clear_signing] matched function {}", function.typed_signature);

    let decoded = decode_arguments(function, calldata)?;
    println!("[clear_signing] decoded {} arguments", decoded.ordered().len());

    if let Some(format_def) = display_formats.get(&function.typed_signature) {
        let (items, mut format_warnings) = apply_display_format(
            format_def,
            &decoded,
            &descriptor.metadata,
            chain_id,
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

#[derive(Debug, Deserialize)]
struct Descriptor {
    context: DescriptorContext,
    #[serde(default)]
    metadata: serde_json::Value,
    #[serde(default)]
    display: DescriptorDisplay,
}

#[derive(Debug, Deserialize)]
struct DescriptorContext {
    contract: DescriptorContract,
}

#[derive(Debug, Deserialize)]
struct DescriptorContract {
    #[serde(default)]
    deployments: Vec<ContractDeployment>,
    #[serde(default)]
    abi: Option<AbiDefinition>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AbiDefinition {
    Inline(Vec<AbiFunction>),
    Reference(String),
}

impl DescriptorContract {
    fn is_bound_to(&self, chain_id: u64, address: &str) -> bool {
        let address = normalize_address(address);
        self.deployments.iter().any(|deployment| {
            deployment.chain_id == chain_id
                && normalize_address(&deployment.address) == address
        })
    }

    fn function_descriptors(
        &self,
    ) -> Result<Vec<FunctionDescriptor>, EngineError> {
        let abi = self.resolve_abi()?;
        Ok(abi.iter().filter_map(FunctionDescriptor::from_abi).collect())
    }

    fn resolve_abi(&self) -> Result<Vec<AbiFunction>, EngineError> {
        match &self.abi {
            Some(AbiDefinition::Inline(functions)) => Ok(functions.clone()),
            Some(AbiDefinition::Reference(reference)) => {
                let abi_json =
                    abi_json_for_reference(reference).ok_or_else(|| {
                        EngineError::DescriptorParse(format!(
                            "unsupported ABI reference '{}'",
                            reference
                        ))
                    })?;
                serde_json::from_str(abi_json).map_err(|err| {
                    EngineError::DescriptorParse(format!(
                        "failed to parse ABI '{}': {}",
                        reference, err
                    ))
                })
            }
            None => Ok(Vec::new()),
        }
    }
}

fn abi_json_for_reference(reference: &str) -> Option<&'static str> {
    match reference {
        "https://github.com/LedgerHQ/ledger-asset-dapps/blob/211e75ed27de3894f592ca73710fa0b72c3ceeae/ethereum/aave/abis/0x7d2768de32b0b80b7a3454c06bdac94a69ddc7a9.abi.json" => {
            Some(AAVE_LPV2_ABI)
        }
        "https://api.etherscan.io/api?module=contract&action=getabi&address=0xd01607c3C5eCABa394D8be377a08590149325722" => {
            Some(AAVE_WETH_GATEWAY_V3_ABI)
        }
        "https://api.etherscan.io/api?module=contract&action=getabi&address=0xef434e4573b90b6ecd4a00f4888381e4d0cc5ccd" => {
            Some(AAVE_LPV3_ABI)
        }
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
struct ContractDeployment {
    #[serde(rename = "chainId")]
    chain_id: u64,
    address: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AbiFunction {
    name: String,
    #[serde(default)]
    inputs: Vec<FunctionInput>,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Debug, Clone, Deserialize)]
struct FunctionInput {
    #[serde(default)]
    name: String,
    #[serde(rename = "type")]
    r#type: String,
}

#[derive(Debug, Clone)]
struct FunctionDescriptor {
    name: String,
    inputs: Vec<FunctionInput>,
    typed_signature: String,
    selector: [u8; 4],
}

impl FunctionDescriptor {
    fn from_abi(function: &AbiFunction) -> Option<Self> {
        if function.kind != "function" {
            return None;
        }

        let typed_signature = typed_signature_for(function);
        let selector = selector_for_signature(&typed_signature);

        Some(Self {
            name: function.name.clone(),
            inputs: function.inputs.clone(),
            typed_signature,
            selector,
        })
    }
}

#[derive(Debug, Default, Deserialize)]
struct DescriptorDisplay {
    #[serde(default)]
    formats: HashMap<String, DisplayFormat>,
}

impl DescriptorDisplay {
    fn format_map(&self) -> HashMap<String, DisplayFormat> {
        self.formats
            .iter()
            .filter_map(|(signature, format)| {
                normalize_signature_key(signature)
                    .map(|normalized| (normalized, format.clone()))
            })
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
struct DisplayFormat {
    intent: String,
    #[serde(default)]
    fields: Vec<DisplayField>,
    #[serde(default)]
    required: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DisplayField {
    path: String,
    label: String,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    params: serde_json::Value,
}

fn apply_display_format(
    format: &DisplayFormat,
    decoded: &DecodedArguments,
    metadata: &serde_json::Value,
    chain_id: u64,
) -> Result<(Vec<DisplayItem>, Vec<String>), EngineError> {
    let mut items = Vec::new();
    let mut warnings = Vec::new();

    for required in &format.required {
        if decoded.get(required).is_none() {
            warnings.push(format!("Missing required argument '{required}'"));
        }
    }

    for field in &format.fields {
        if let Some(value) = decoded.get(&field.path) {
            let rendered =
                render_field(field, value, decoded, metadata, chain_id)?;
            items.push(DisplayItem {
                label: field.label.clone(),
                value: rendered,
            });
        } else {
            warnings.push(format!(
                "No value found for field path '{}'",
                field.path
            ));
        }
    }

    Ok((items, warnings))
}

fn render_field(
    field: &DisplayField,
    value: &ArgumentValue,
    decoded: &DecodedArguments,
    metadata: &serde_json::Value,
    chain_id: u64,
) -> Result<String, EngineError> {
    match field.format.as_deref() {
        Some("date") => Ok(format_date(value)),
        Some("tokenAmount") => {
            format_token_amount(field, value, decoded, metadata, chain_id)
        }
        Some("amount") => Ok(format_native_amount(value)),
        Some("address") | Some("addressName") => Ok(format_address(value)),
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
    field: &DisplayField,
    value: &ArgumentValue,
    decoded: &DecodedArguments,
    metadata: &serde_json::Value,
    chain_id: u64,
) -> Result<String, EngineError> {
    let ArgumentValue::Uint(amount) = value else {
        return Ok(value.default_string());
    };

    if let Some(message) = token_amount_message(field, amount, metadata) {
        return Ok(message);
    }

    let token_meta = resolve_token_meta(field, decoded, chain_id)?;
    let formatted_amount =
        format_amount_with_decimals(amount, token_meta.decimals);
    Ok(format!("{} {}", formatted_amount, token_meta.symbol))
}

fn format_address(value: &ArgumentValue) -> String {
    let Some(bytes) = value.as_address() else {
        return value.default_string();
    };
    to_checksum_address(bytes)
}

fn format_number(value: &ArgumentValue) -> String {
    let Some(number) = value.as_uint() else {
        return value.default_string();
    };
    number.to_string()
}

fn format_native_amount(value: &ArgumentValue) -> String {
    let ArgumentValue::Uint(amount) = value else {
        return value.default_string();
    };
    let formatted = format_amount_with_decimals(amount, 18);
    format!("{} ETH", formatted)
}

fn token_amount_message(
    field: &DisplayField,
    amount: &BigUint,
    metadata: &serde_json::Value,
) -> Option<String> {
    let threshold_ptr =
        field.params.get("threshold").and_then(|value| value.as_str())?;
    let message =
        field.params.get("message").and_then(|value| value.as_str())?;

    let threshold = resolve_metadata_biguint(metadata, threshold_ptr)?;
    if amount == &threshold {
        Some(message.to_string())
    } else {
        None
    }
}

fn resolve_token_meta(
    field: &DisplayField,
    decoded: &DecodedArguments,
    chain_id: u64,
) -> Result<TokenMeta, EngineError> {
    if let Some(token) =
        field.params.get("token").and_then(|value| value.as_str())
    {
        return token_registry::lookup_token_by_caip19(token).ok_or_else(
            || {
                EngineError::TokenRegistry(format!(
                    "token registry missing entry for {}",
                    token
                ))
            },
        );
    }

    if let Some(token_path) =
        field.params.get("tokenPath").and_then(|value| value.as_str())
    {
        let token_value = decoded.get(token_path).ok_or_else(|| {
            EngineError::TokenRegistry(format!(
                "token path '{}' not found for field '{}'",
                token_path, field.path
            ))
        })?;

        let address_bytes = token_value.as_address().ok_or_else(|| {
            EngineError::TokenRegistry(format!(
                "token path '{}' is not an address for field '{}'",
                token_path, field.path
            ))
        })?;

        let address = format!("0x{}", hex::encode(address_bytes));
        return token_registry::lookup_token(chain_id, &address).ok_or_else(
            || {
                EngineError::TokenRegistry(format!(
                    "token registry missing entry for chain {} and address {}",
                    chain_id, address
                ))
            },
        );
    }

    Err(EngineError::TokenRegistry(format!(
        "missing token lookup parameters for field '{}'",
        field.path
    )))
}

fn format_amount_with_decimals(amount: &BigUint, decimals: u8) -> String {
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

fn add_thousand_separators(value: &str) -> String {
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

fn resolve_metadata_value<'a>(
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

fn parse_biguint(text: &str) -> Option<BigUint> {
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

fn decode_arguments(
    function: &FunctionDescriptor,
    calldata: &[u8],
) -> Result<DecodedArguments, EngineError> {
    let expected_len = 4 + function.inputs.len() * 32;
    if calldata.len() < expected_len {
        return Err(EngineError::Calldata(format!(
            "calldata length {} too small for {} arguments",
            calldata.len(),
            function.inputs.len()
        )));
    }

    let mut decoded = DecodedArguments::new();
    for (index, input) in function.inputs.iter().enumerate() {
        let start = 4 + index * 32;
        let end = start + 32;
        let mut word = [0u8; 32];
        word.copy_from_slice(&calldata[start..end]);
        let value = decode_word(&input.r#type, &word);
        let name = if input.name.trim().is_empty() {
            None
        } else {
            Some(input.name.clone())
        };
        decoded.push(name, index, value, word);
    }

    Ok(decoded)
}

fn decode_word(kind: &str, word: &[u8; 32]) -> ArgumentValue {
    match kind {
        t if t.starts_with("uint") => {
            ArgumentValue::Uint(BigUint::from_bytes_be(word))
        }
        "address" => {
            let mut bytes = [0u8; 20];
            bytes.copy_from_slice(&word[12..]);
            ArgumentValue::Address(bytes)
        }
        _ => ArgumentValue::Raw(*word),
    }
}

#[derive(Debug, Clone)]
struct DecodedArgument {
    index: usize,
    name: Option<String>,
    value: ArgumentValue,
    word: [u8; 32],
}

impl DecodedArgument {
    fn display_label(&self) -> String {
        self.name
            .clone()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| format!("arg{}", self.index))
    }

    fn raw_word_hex(&self) -> String {
        format!("0x{}", hex::encode(self.word))
    }
}

#[derive(Debug, Clone)]
struct DecodedArguments {
    ordered: Vec<DecodedArgument>,
    index_by_name: HashMap<String, usize>,
}

impl DecodedArguments {
    fn new() -> Self {
        Self { ordered: Vec::new(), index_by_name: HashMap::new() }
    }

    fn push(
        &mut self,
        name: Option<String>,
        index: usize,
        value: ArgumentValue,
        word: [u8; 32],
    ) {
        let entry_index = self.ordered.len();
        if let Some(ref name) = name {
            self.index_by_name.insert(name.clone(), entry_index);
        }
        self.index_by_name.insert(format!("arg{}", index), entry_index);
        self.ordered.push(DecodedArgument { index, name, value, word });
    }

    fn get(&self, key: &str) -> Option<&ArgumentValue> {
        self.index_by_name
            .get(key)
            .and_then(|&idx| self.ordered.get(idx))
            .map(|entry| &entry.value)
    }

    fn ordered(&self) -> &[DecodedArgument] {
        &self.ordered
    }
}

#[derive(Debug, Clone)]
enum ArgumentValue {
    Address([u8; 20]),
    Uint(BigUint),
    Raw([u8; 32]),
}

impl ArgumentValue {
    fn default_string(&self) -> String {
        match self {
            ArgumentValue::Address(bytes) => {
                format!("0x{}", hex::encode(bytes))
            }
            ArgumentValue::Uint(value) => value.to_string(),
            ArgumentValue::Raw(bytes) => format!("0x{}", hex::encode(bytes)),
        }
    }

    fn as_uint(&self) -> Option<&BigUint> {
        if let ArgumentValue::Uint(value) = self {
            Some(value)
        } else {
            None
        }
    }

    fn as_address(&self) -> Option<&[u8; 20]> {
        if let ArgumentValue::Address(bytes) = self {
            Some(bytes)
        } else {
            None
        }
    }
}

fn normalize_signature_key(signature: &str) -> Option<String> {
    let open_paren = signature.find('(')?;
    let close_paren = signature.rfind(')')?;
    let name = signature[..open_paren].trim();
    let params = &signature[open_paren + 1..close_paren];
    let mut types = Vec::new();
    if !params.trim().is_empty() {
        for param in params.split(',') {
            let trimmed = param.trim();
            if trimmed.is_empty() {
                continue;
            }
            let ty = trimmed.split_whitespace().next().unwrap_or(trimmed);
            types.push(ty.trim().to_string());
        }
    }
    Some(format!("{}({})", name, types.join(",")))
}

fn typed_signature_for(function: &AbiFunction) -> String {
    let mut params = Vec::with_capacity(function.inputs.len());
    for input in &function.inputs {
        params.push(input.r#type.trim().to_string());
    }
    format!("{}({})", function.name.trim(), params.join(","))
}

fn selector_for_signature(signature: &str) -> [u8; 4] {
    let mut hasher = Keccak::v256();
    hasher.update(signature.as_bytes());
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    [output[0], output[1], output[2], output[3]]
}

fn format_selector_hex(selector: &[u8; 4]) -> String {
    format!("0x{}", hex::encode(selector))
}

fn normalize_address(address: &str) -> String {
    address.trim().to_ascii_lowercase()
}

fn to_checksum_address(bytes: &[u8; 20]) -> String {
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

    #[test]
    fn normalize_signature_without_param_names() {
        assert_eq!(
            normalize_signature_key("foo(uint256,uint256)"),
            Some("foo(uint256,uint256)".to_string())
        );
    }

    #[test]
    fn normalize_signature_with_param_names() {
        assert_eq!(
            normalize_signature_key("foo(uint256 amount, address to)"),
            Some("foo(uint256,address)".to_string())
        );
    }
}
