//! Descriptor parsing and token lookup utilities for clear signing.

use {
    super::resolver::ResolvedDescriptor, num_bigint::BigUint,
    serde::Deserialize, serde_json::Value as JsonValue,
    std::collections::HashMap, thiserror::Error,
};

#[derive(Debug, Error)]
pub enum DescriptorError {
    #[error("descriptor parse error: {0}")]
    Parse(String),
    #[error("calldata decode error: {0}")]
    Calldata(String),
}

pub(crate) fn build_descriptor(
    resolved: &ResolvedDescriptor<'_>,
) -> Result<Descriptor, DescriptorError> {
    let mut descriptor_value: JsonValue =
        serde_json::from_str(resolved.descriptor_json)
            .map_err(|err| DescriptorError::Parse(err.to_string()))?;

    for include_json in &resolved.includes {
        let include_value: JsonValue = serde_json::from_str(include_json)
            .map_err(|err| DescriptorError::Parse(err.to_string()))?;
        merge_include(&mut descriptor_value, include_value);
    }

    if let Some(object) = descriptor_value.as_object_mut() {
        object.remove("includes");
    }

    if let Some(abi_json) = resolved.abi_json {
        if needs_abi_injection(&descriptor_value) {
            let abi_value: JsonValue = serde_json::from_str(abi_json)
                .map_err(|err| DescriptorError::Parse(err.to_string()))?;
            inject_abi(&mut descriptor_value, abi_value);
        }
    }

    serde_json::from_value(descriptor_value)
        .map_err(|err| DescriptorError::Parse(err.to_string()))
}

fn needs_abi_injection(descriptor_value: &JsonValue) -> bool {
    let abi_value = descriptor_value
        .get("context")
        .and_then(|context| context.get("contract"))
        .and_then(|contract| contract.get("abi"));
    match abi_value {
        Some(value) if value.is_array() || value.is_object() => false,
        Some(value) if value.is_null() => true,
        None => true,
        Some(_) => true,
    }
}

fn inject_abi(descriptor_value: &mut JsonValue, abi_value: JsonValue) {
    if let Some(contract) = descriptor_value
        .get_mut("context")
        .and_then(|context| context.as_object_mut())
        .and_then(|context_obj| context_obj.get_mut("contract"))
        .and_then(|contract| contract.as_object_mut())
    {
        contract.insert("abi".to_string(), abi_value);
    }
}

fn merge_include(target: &mut JsonValue, include: JsonValue) {
    if let (JsonValue::Object(target_map), JsonValue::Object(include_map)) =
        (target, include)
    {
        for (key, value) in include_map {
            match target_map.get_mut(&key) {
                Some(existing) => {
                    if existing.is_object() && value.is_object() {
                        merge_include(existing, value);
                    }
                }
                None => {
                    target_map.insert(key, value);
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Descriptor {
    pub context: DescriptorContext,
    #[serde(default)]
    pub metadata: JsonValue,
    #[serde(default)]
    pub display: DescriptorDisplay,
}

#[derive(Debug, Deserialize)]
pub struct DescriptorContext {
    #[serde(rename = "$id")]
    #[serde(default)]
    pub id: Option<String>,
    pub contract: DescriptorContract,
}

#[derive(Debug, Deserialize)]
pub struct DescriptorContract {
    #[serde(default)]
    pub deployments: Vec<ContractDeployment>,
    #[serde(default)]
    pub abi: Option<AbiDefinition>,
}

impl DescriptorContract {
    pub fn is_bound_to(&self, chain_id: u64, address: &str) -> bool {
        let address = normalize_address(address);
        self.deployments.iter().any(|deployment| {
            deployment.chain_id == chain_id
                && normalize_address(&deployment.address) == address
        })
    }

    pub fn function_descriptors(
        &self,
    ) -> Result<Vec<FunctionDescriptor>, DescriptorError> {
        let abi = self.resolve_abi()?;
        Ok(abi.iter().filter_map(FunctionDescriptor::from_abi).collect())
    }

    fn resolve_abi(&self) -> Result<Vec<AbiFunction>, DescriptorError> {
        match &self.abi {
            Some(AbiDefinition::Inline(functions)) => Ok(functions.clone()),
            Some(AbiDefinition::Reference(reference)) => {
                Err(DescriptorError::Parse(format!(
                    "unsupported ABI reference '{}'",
                    reference
                )))
            }
            None => Ok(Vec::new()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AbiDefinition {
    Inline(Vec<AbiFunction>),
    Reference(String),
}

#[derive(Debug, Deserialize)]
pub struct ContractDeployment {
    #[serde(rename = "chainId")]
    pub chain_id: u64,
    pub address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AbiFunction {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub inputs: Vec<FunctionInput>,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FunctionInput {
    #[serde(default)]
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "internalType")]
    #[serde(default)]
    pub internal_type: Option<String>,
    #[serde(default)]
    pub components: Vec<FunctionInput>,
}

#[derive(Debug, Clone)]
pub struct FunctionDescriptor {
    pub inputs: Vec<FunctionInput>,
    pub typed_signature: String,
    pub selector: [u8; 4],
}

impl FunctionDescriptor {
    pub fn from_abi(function: &AbiFunction) -> Option<Self> {
        if function.kind != "function" {
            return None;
        }

        let typed_signature = typed_signature_for(function);
        let selector = selector_for_signature(&typed_signature);

        Some(Self {
            inputs: function.inputs.clone(),
            typed_signature,
            selector,
        })
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
        params.push(type_signature_for_input(input));
    }
    format!("{}({})", function.name.trim(), params.join(","))
}

fn selector_for_signature(signature: &str) -> [u8; 4] {
    use tiny_keccak::{Hasher, Keccak};

    let mut hasher = Keccak::v256();
    hasher.update(signature.as_bytes());
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    [output[0], output[1], output[2], output[3]]
}

fn type_signature_for_input(input: &FunctionInput) -> String {
    let ty = input.r#type.trim();
    if let Some(stripped) = ty.strip_prefix("tuple") {
        let suffix = stripped;
        let nested = input
            .components
            .iter()
            .map(type_signature_for_input)
            .collect::<Vec<_>>()
            .join(",");
        format!("({}){}", nested, suffix)
    } else {
        ty.to_string()
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct DescriptorDisplay {
    #[serde(default)]
    definitions: HashMap<String, DisplayField>,
    #[serde(default)]
    formats: HashMap<String, DisplayFormat>,
}

impl DescriptorDisplay {
    pub fn format_map(&self) -> HashMap<String, DisplayFormat> {
        self.formats
            .iter()
            .filter_map(|(signature, format)| {
                normalize_signature_key(signature)
                    .map(|normalized| (normalized, format.clone()))
            })
            .collect()
    }

    pub fn definitions(&self) -> &HashMap<String, DisplayField> {
        &self.definitions
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DisplayFormat {
    #[serde(default)]
    pub intent: String,
    #[serde(default, rename = "interpolatedIntent")]
    pub interpolated_intent: Option<String>,
    #[serde(default)]
    pub fields: Vec<DisplayField>,
    #[serde(default)]
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DisplayField {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub params: JsonValue,
    #[serde(rename = "$ref")]
    #[serde(default)]
    pub reference: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EffectiveField {
    pub path: String,
    pub label: String,
    pub format: Option<String>,
    pub params: JsonValue,
}

pub fn resolve_effective_field(
    field: &DisplayField,
    definitions: &HashMap<String, DisplayField>,
    warnings: &mut Vec<String>,
) -> Option<EffectiveField> {
    let mut path = field.path.clone();
    let mut label = field.label.clone();
    let mut format = field.format.clone();
    let mut params = field.params.clone();

    if let Some(reference) = &field.reference {
        if let Some(name) = extract_definition_name(reference) {
            if let Some(def) = definitions.get(name) {
                if path.is_none() {
                    path = def.path.clone();
                }
                if label.is_none() {
                    label = def.label.clone();
                }
                if format.is_none() {
                    format = def.format.clone();
                }
                params = merge_params(&def.params, &params);
            } else {
                warnings.push(format!(
                    "Unknown display definition reference '{}'",
                    reference
                ));
            }
        } else {
            warnings.push(format!(
                "Unsupported display definition reference '{}'",
                reference
            ));
        }
    }

    let path = path?;
    let label = label.unwrap_or_else(|| path.clone());

    Some(EffectiveField { path, label, format, params })
}

pub fn extract_definition_name(reference: &str) -> Option<&str> {
    reference.strip_prefix("$.display.definitions.")
}

pub fn merge_params(base: &JsonValue, overlay: &JsonValue) -> JsonValue {
    match (base, overlay) {
        (JsonValue::Object(base_map), JsonValue::Object(overlay_map)) => {
            let mut merged = base_map.clone();
            for (key, value) in overlay_map {
                merged.insert(key.clone(), value.clone());
            }
            JsonValue::Object(merged)
        }
        (_, JsonValue::Null) => base.clone(),
        (_, overlay_value) if overlay_value.is_null() => base.clone(),
        (_, overlay_value) => overlay_value.clone(),
    }
}

#[derive(Debug, Clone)]
pub struct DecodedArgument {
    pub index: usize,
    pub name: Option<String>,
    pub value: ArgumentValue,
    pub word: [u8; 32],
}

impl DecodedArgument {
    pub fn display_label(&self) -> String {
        self.name
            .clone()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| format!("arg{}", self.index))
    }

    pub fn raw_word_hex(&self) -> String {
        format!("0x{}", hex::encode(self.word))
    }
}

#[derive(Debug, Clone)]
pub struct DecodedArguments {
    ordered: Vec<DecodedArgument>,
    index_by_name: HashMap<String, usize>,
}

impl DecodedArguments {
    pub fn new() -> Self {
        Self { ordered: Vec::new(), index_by_name: HashMap::new() }
    }

    pub fn with_value(
        mut self,
        value: Option<&[u8]>,
    ) -> Result<Self, DescriptorError> {
        if let Some(raw) = value {
            if raw.len() > 32 {
                return Err(DescriptorError::Calldata(
                    "call value must be at most 32 bytes".to_string(),
                ));
            }
            let mut word = [0u8; 32];
            let start = 32 - raw.len();
            word[start..].copy_from_slice(raw);
            let amount = BigUint::from_bytes_be(&word);
            self.push(
                Some("@value".to_string()),
                self.ordered.len(),
                ArgumentValue::Uint(amount),
                word,
            );
        }
        Ok(self)
    }

    pub fn push(
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

    pub fn get(&self, key: &str) -> Option<&ArgumentValue> {
        self.index_by_name
            .get(key)
            .and_then(|&idx| self.ordered.get(idx))
            .map(|entry| &entry.value)
    }

    pub fn ordered(&self) -> &[DecodedArgument] {
        &self.ordered
    }
}

#[derive(Debug, Clone)]
pub enum ArgumentValue {
    Address([u8; 20]),
    Uint(BigUint),
    Raw([u8; 32]),
}

impl ArgumentValue {
    pub fn default_string(&self) -> String {
        match self {
            ArgumentValue::Address(bytes) => {
                format!("0x{}", hex::encode(bytes))
            }
            ArgumentValue::Uint(value) => value.to_string(),
            ArgumentValue::Raw(bytes) => format!("0x{}", hex::encode(bytes)),
        }
    }

    pub fn as_uint(&self) -> Option<&BigUint> {
        if let ArgumentValue::Uint(value) = self { Some(value) } else { None }
    }

    pub fn as_address(&self) -> Option<&[u8; 20]> {
        if let ArgumentValue::Address(bytes) = self {
            Some(bytes)
        } else {
            None
        }
    }
}

pub fn decode_arguments(
    function: &FunctionDescriptor,
    calldata: &[u8],
) -> Result<DecodedArguments, DescriptorError> {
    let total_words: usize =
        function.inputs.iter().map(argument_word_count).sum();
    let expected_len = 4 + total_words * 32;
    if calldata.len() < expected_len {
        return Err(DescriptorError::Calldata(format!(
            "calldata length {} too small for {} arguments",
            calldata.len(),
            total_words
        )));
    }

    let mut decoded = DecodedArguments::new();
    let mut cursor = 4;
    let mut global_index = 0;
    for input in &function.inputs {
        decode_input(
            input,
            calldata,
            &mut cursor,
            &mut decoded,
            None,
            &mut global_index,
        )?;
    }

    Ok(decoded)
}

fn decode_input(
    input: &FunctionInput,
    calldata: &[u8],
    cursor: &mut usize,
    decoded: &mut DecodedArguments,
    prefix: Option<String>,
    global_index: &mut usize,
) -> Result<(), DescriptorError> {
    if input.r#type.starts_with("tuple") && !input.components.is_empty() {
        let base_prefix = if let Some(existing) = prefix {
            if input.name.trim().is_empty() {
                existing
            } else {
                format!("{}.{}", existing, input.name.trim())
            }
        } else if input.name.trim().is_empty() {
            String::new()
        } else {
            input.name.trim().to_string()
        };
        let next_prefix = if base_prefix.is_empty() {
            None
        } else {
            Some(base_prefix.clone())
        };
        for component in &input.components {
            decode_input(
                component,
                calldata,
                cursor,
                decoded,
                next_prefix.clone(),
                global_index,
            )?;
        }
        Ok(())
    } else {
        let start = *cursor;
        let end = start + 32;
        if end > calldata.len() {
            return Err(DescriptorError::Calldata(format!(
                "calldata length {} too small while decoding argument '{}'",
                calldata.len(),
                input.name
            )));
        }
        let mut word = [0u8; 32];
        word.copy_from_slice(&calldata[start..end]);
        *cursor = end;

        let value =
            decode_word(&input.r#type, input.internal_type.as_deref(), &word);
        let name = argument_name(prefix, input);
        decoded.push(name, *global_index, value, word);
        *global_index += 1;
        Ok(())
    }
}

fn argument_word_count(input: &FunctionInput) -> usize {
    if input.r#type.starts_with("tuple") && !input.components.is_empty() {
        input.components.iter().map(argument_word_count).sum()
    } else {
        1
    }
}

fn argument_name(
    prefix: Option<String>,
    input: &FunctionInput,
) -> Option<String> {
    let trimmed = input.name.trim();
    if let Some(prefix) = prefix {
        if trimmed.is_empty() {
            Some(prefix)
        } else {
            Some(format!("{}.{}", prefix, trimmed))
        }
    } else if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn decode_word(
    kind: &str,
    internal_type: Option<&str>,
    word: &[u8; 32],
) -> ArgumentValue {
    if internal_type_is_address(internal_type, kind) || kind == "address" {
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&word[12..]);
        return ArgumentValue::Address(bytes);
    }

    if kind.starts_with("uint") {
        return ArgumentValue::Uint(BigUint::from_bytes_be(word));
    }

    ArgumentValue::Raw(*word)
}

fn internal_type_is_address(internal_type: Option<&str>, kind: &str) -> bool {
    let Some(alias) = internal_type else {
        return false;
    };
    let normalized = alias.trim();
    if normalized.is_empty() {
        return false;
    }
    if normalized.eq_ignore_ascii_case("address") {
        return true;
    }
    if normalized
        .rsplit([' ', '.', ':'])
        .next()
        .map(|segment| segment.eq_ignore_ascii_case("address"))
        .unwrap_or(false)
    {
        return true;
    }
    normalized == "Address" && kind.starts_with("uint")
}

fn normalize_address(address: &str) -> String {
    address.trim().to_ascii_lowercase()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenLookupKey(String);

impl TokenLookupKey {
    pub fn from_caip19(value: impl AsRef<str>) -> Self {
        TokenLookupKey(normalize_caip19(value.as_ref()))
    }

    pub fn from_erc20(chain_id: u64, address: &str) -> Self {
        let normalized = normalize_address(address);
        let key = format!("eip155:{}/erc20:{}", chain_id, normalized);
        TokenLookupKey::from_caip19(key)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum TokenLookupError {
    #[error("display field '{field}' missing token configuration")]
    MissingToken { field: String },
    #[error("token path '{path}' not found for field '{field}'")]
    MissingTokenPath { path: String, field: String },
    #[error("token path '{path}' is not an address for field '{field}'")]
    TokenPathNotAddress { path: String, field: String },
}

pub fn determine_token_key(
    field: &EffectiveField,
    decoded: &DecodedArguments,
    chain_id: u64,
    contract_address: &str,
) -> Result<TokenLookupKey, TokenLookupError> {
    if let Some(token) =
        field.params.get("token").and_then(|value| value.as_str())
    {
        return Ok(TokenLookupKey::from_caip19(token));
    }

    if let Some(token_path) =
        field.params.get("tokenPath").and_then(|value| value.as_str())
    {
        let address = if token_path == "@.to" {
            normalize_address(contract_address)
        } else {
            let token_value = decoded.get(token_path).ok_or_else(|| {
                TokenLookupError::MissingTokenPath {
                    path: token_path.to_string(),
                    field: field.path.clone(),
                }
            })?;

            let address_bytes = token_value.as_address().ok_or_else(|| {
                TokenLookupError::TokenPathNotAddress {
                    path: token_path.to_string(),
                    field: field.path.clone(),
                }
            })?;

            let addr = format!("0x{}", hex::encode(address_bytes));
            normalize_address(&addr)
        };

        return Ok(TokenLookupKey::from_erc20(chain_id, &address));
    }

    Err(TokenLookupError::MissingToken { field: field.path.clone() })
}

pub fn native_token_key(
    chain_id: u64,
) -> Result<TokenLookupKey, TokenLookupError> {
    let slip44 = native_slip44_code(chain_id).ok_or_else(|| {
        TokenLookupError::MissingToken {
            field: format!("native token for chain {}", chain_id),
        }
    })?;
    let caip19 = format!("eip155:{}/slip44:{}", chain_id, slip44);
    let caip19 = normalize_caip19(&caip19);
    Ok(TokenLookupKey::from_caip19(caip19))
}

fn native_slip44_code(chain_id: u64) -> Option<u32> {
    match chain_id {
        1 | 10 | 42161 | 8453 => Some(60),
        _ => None,
    }
}

fn normalize_caip19(input: &str) -> String {
    input.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caip19_normalization() {
        use serde_json::json;

        // Test with mixed case and whitespace in CAIP-19 string
        let params = json!({
            "token": "  EIP155:1/ERC20:0xA0B86991C6218b36c1D19D4a2e9Eb0cE3606eB48  "
        });

        let field = EffectiveField {
            path: "test".to_string(),
            label: "Test".to_string(),
            format: Some("tokenAmount".to_string()),
            params,
        };

        let decoded = DecodedArguments::new();
        let result = determine_token_key(&field, &decoded, 1, "0xtest");

        assert!(result.is_ok());
        let key = result.unwrap();

        // Should be normalized to lowercase with whitespace trimmed
        assert_eq!(
            key,
            TokenLookupKey::from_caip19(
                "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
            )
        );
    }

    #[test]
    fn test_native_token_key_constructs_caip19() {
        let result = native_token_key(1);
        assert!(result.is_ok());
        let key = result.unwrap();
        assert_eq!(key, TokenLookupKey::from_caip19("eip155:1/slip44:60"));
    }

    #[test]
    fn test_native_token_key_unsupported_chain() {
        let result = native_token_key(999);
        assert!(result.is_err());
    }
}
