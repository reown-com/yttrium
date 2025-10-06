use std::collections::HashMap;

use num_bigint::BigUint;
use serde::Deserialize;
use thiserror::Error;
use time::{macros::format_description, OffsetDateTime};
use tiny_keccak::{Hasher, Keccak};

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

    let functions = descriptor.context.contract.function_descriptors();
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
        println!(
            "[clear_signing] no ABI match for selector {}",
            selector_hex
        );
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

    println!(
        "[clear_signing] matched function {}",
        function.typed_signature
    );

    let decoded = decode_arguments(function, calldata)?;
    println!(
        "[clear_signing] decoded {} arguments",
        decoded.ordered().len()
    );

    if let Some(format_def) = display_formats.get(&function.typed_signature) {
        let (items, mut format_warnings) =
            apply_display_format(format_def, &decoded);
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
    abi: Vec<AbiFunction>,
}

impl DescriptorContract {
    fn is_bound_to(&self, chain_id: u64, address: &str) -> bool {
        let address = normalize_address(address);
        self.deployments.iter().any(|deployment| {
            deployment.chain_id == chain_id
                && normalize_address(&deployment.address) == address
        })
    }

    fn function_descriptors(&self) -> Vec<FunctionDescriptor> {
        self.abi.iter().filter_map(FunctionDescriptor::from_abi).collect()
    }
}

#[derive(Debug, Deserialize)]
struct ContractDeployment {
    #[serde(rename = "chainId")]
    chain_id: u64,
    address: String,
}

#[derive(Debug, Deserialize)]
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
) -> (Vec<DisplayItem>, Vec<String>) {
    let mut items = Vec::new();
    let mut warnings = Vec::new();

    for required in &format.required {
        if decoded.get(required).is_none() {
            warnings.push(format!("Missing required argument '{required}'"));
        }
    }

    for field in &format.fields {
        if let Some(value) = decoded.get(&field.path) {
            let rendered = render_field(field, value);
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

    (items, warnings)
}

fn render_field(field: &DisplayField, value: &ArgumentValue) -> String {
    match field.format.as_deref() {
        Some("date") => format_date(value),
        Some("tokenAmount") => format_token_amount(value),
        Some("address") | Some("addressName") => format_address(value),
        Some("number") => format_number(value),
        _ => value.default_string(),
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

fn format_token_amount(value: &ArgumentValue) -> String {
    let ArgumentValue::Uint(amount) = value else {
        return value.default_string();
    };

    let decimals = BigUint::from(10u32).pow(18);
    let integer = amount / &decimals;
    let remainder = amount % &decimals;
    if remainder == BigUint::from(0u32) {
        return integer.to_string();
    }

    let mut fractional = format!("{:018}", remainder);
    while fractional.ends_with('0') {
        fractional.pop();
    }

    format!("{}.{}", integer.to_string(), fractional)
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
