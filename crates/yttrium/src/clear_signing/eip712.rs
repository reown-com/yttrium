use {
    super::{
        descriptor::{
            DisplayField, DisplayFormat, EffectiveField,
            resolve_effective_field,
        },
        engine::{
            DisplayItem, DisplayModel, format_amount_with_decimals,
            interpolate_template, parse_biguint, resolve_metadata_value,
        },
        resolver::{self, ResolvedTypedDescriptor},
        token_registry::lookup_token_by_caip19,
    },
    num_bigint::BigUint,
    serde::Deserialize,
    serde_json::Value,
    std::collections::HashMap,
    thiserror::Error,
    time::{OffsetDateTime, macros::format_description},
};

#[derive(Debug, Clone, Deserialize)]
pub struct TypedData {
    #[serde(rename = "types")]
    pub types: HashMap<String, Vec<TypeMember>>,
    #[serde(rename = "primaryType")]
    pub primary_type: String,
    pub domain: Value,
    pub message: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TypeMember {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
}

#[derive(Debug, Error)]
pub enum Eip712Error {
    #[error("resolver error: {0}")]
    Resolver(String),
    #[error("descriptor parse error: {0}")]
    DescriptorParse(String),
    #[error("typed data error: {0}")]
    TypedData(String),
    #[error("token registry error: {0}")]
    TokenRegistry(String),
}

pub fn format_typed_data(
    data: &TypedData,
) -> Result<DisplayModel, Eip712Error> {
    let chain_id = extract_chain_id(&data.domain)?;
    let verifying_contract = extract_verifying_contract(&data.domain)?;

    let resolved = resolver::resolve_typed(chain_id, &verifying_contract)
        .map_err(|err| Eip712Error::Resolver(err.to_string()))?;
    let descriptor = parse_descriptor(&resolved)?;
    let mut warnings = Vec::new();

    if let Some(context) = descriptor.context.as_ref()
        && !context.eip712.deployments.iter().any(|deployment| {
            deployment.chain_id == chain_id
                && deployment.address.eq_ignore_ascii_case(&verifying_contract)
        })
    {
        warnings.push(format!(
            "Descriptor deployment mismatch for chain {} \
             and address {}",
            chain_id, verifying_contract
        ));
    }

    let Some(format) = descriptor.display.formats.get(&data.primary_type)
    else {
        return Err(Eip712Error::TypedData(format!(
            "No display format for primary type {}",
            data.primary_type
        )));
    };

    let mut items = Vec::new();
    let address_book = &resolved.address_book;
    let mut rendered_values: HashMap<String, String> = HashMap::new();

    for required in &format.required {
        if get_value(&data.message, required).is_none() {
            warnings.push(format!("Missing required field '{required}'"));
        }
    }

    for field in &format.fields {
        let Some(effective) = resolve_effective_field(
            field,
            &descriptor.display.definitions,
            &mut warnings,
        ) else {
            continue;
        };
        let Some(value) = get_value(&data.message, &effective.path) else {
            warnings.push(format!(
                "No value found for field path '{}'",
                effective.path
            ));
            continue;
        };

        let rendered = render_field(
            &effective,
            value,
            &data.message,
            &descriptor.metadata,
            chain_id,
            address_book,
            &mut warnings,
        )?;
        rendered_values.insert(effective.path.clone(), rendered.clone());
        items.push(DisplayItem { label: effective.label, value: rendered });
    }

    let interpolated_intent =
        if let Some(template) = format.interpolated_intent.as_deref() {
            match interpolate_template(template, &rendered_values) {
                Ok(value) => Some(value),
                Err(message) => {
                    warnings.push(message);
                    None
                }
            }
        } else {
            None
        };

    Ok(DisplayModel {
        intent: format.intent.clone(),
        interpolated_intent,
        items,
        warnings,
        raw: None,
    })
}

fn parse_descriptor(
    resolved: &ResolvedTypedDescriptor<'_>,
) -> Result<TypedDescriptor, Eip712Error> {
    let descriptor_value = resolver::merged_descriptor_value(
        resolved.descriptor_json,
        &resolved.includes,
    )
    .map_err(|err| Eip712Error::DescriptorParse(err.to_string()))?;

    serde_json::from_value(descriptor_value)
        .map_err(|err| Eip712Error::DescriptorParse(err.to_string()))
}

#[derive(Debug, Deserialize)]
struct TypedDescriptor {
    #[serde(default)]
    context: Option<TypedContext>,
    #[serde(default)]
    metadata: Value,
    #[serde(default)]
    display: TypedDisplay,
}

#[derive(Debug, Deserialize)]
struct TypedContext {
    eip712: TypedEip712Context,
}

#[derive(Debug, Deserialize)]
struct TypedEip712Context {
    #[serde(default)]
    deployments: Vec<Deployment>,
}

#[derive(Debug, Deserialize)]
struct Deployment {
    #[serde(rename = "chainId")]
    chain_id: u64,
    address: String,
}

#[derive(Debug, Default, Deserialize)]
struct TypedDisplay {
    #[serde(default)]
    definitions: HashMap<String, DisplayField>,
    #[serde(default)]
    formats: HashMap<String, DisplayFormat>,
}

fn render_field(
    field: &EffectiveField,
    value: &Value,
    message: &Value,
    metadata: &Value,
    chain_id: u64,
    address_book: &HashMap<String, String>,
    warnings: &mut Vec<String>,
) -> Result<String, Eip712Error> {
    match field.format.as_deref() {
        Some("tokenAmount") => format_token_amount(
            field, value, message, metadata, chain_id, warnings,
        ),
        Some("date") => Ok(format_date(value)),
        Some("number") => Ok(format_number(value)),
        Some("address") | Some("addressName") => {
            Ok(format_address(value, address_book))
        }
        Some("enum") => format_enum(field, value, metadata),
        Some("raw") => Ok(format_raw(value)),
        _ => Ok(format_raw(value)),
    }
}

fn format_token_amount(
    field: &EffectiveField,
    value: &Value,
    message: &Value,
    metadata: &Value,
    chain_id: u64,
    warnings: &mut Vec<String>,
) -> Result<String, Eip712Error> {
    let field_path = field.path.clone();
    let amount = parse_biguint_from_value(value).ok_or_else(|| {
        Eip712Error::TypedData("token amount is not a number".to_string())
    })?;

    if let Some(token_path) =
        field.params.get("tokenPath").and_then(|value| value.as_str())
    {
        let token_value = get_value(message, token_path).ok_or_else(|| {
            Eip712Error::TypedData(format!(
                "token path '{}' not found for field '{}'",
                token_path, field_path
            ))
        })?;

        let token_address =
            extract_address_value(token_value).ok_or_else(|| {
                Eip712Error::TypedData(format!(
                    "token path '{}' is not an address for field '{}'",
                    token_path, field_path
                ))
            })?;

        let caip19 = format!(
            "eip155:{}/erc20:{}",
            chain_id,
            token_address.to_ascii_lowercase()
        );
        let Some(meta) = lookup_token_by_caip19(&caip19) else {
            warnings.push(format!(
                "Token registry missing entry for chain {} and address {}",
                chain_id, token_address
            ));
            return Ok(format_raw(value));
        };

        if let Some(message) = token_amount_message(field, &amount, metadata) {
            return Ok(format!("{} {}", message, meta.symbol));
        }

        let formatted = format_amount_with_decimals(&amount, meta.decimals);
        return Ok(format!("{} {}", formatted, meta.symbol));
    }

    Err(Eip712Error::TypedData(format!(
        "missing token lookup parameters for field '{}'",
        field_path
    )))
}

fn token_amount_message(
    field: &EffectiveField,
    amount: &BigUint,
    metadata: &Value,
) -> Option<String> {
    let threshold_spec =
        field.params.get("threshold").and_then(|value| value.as_str())?;
    let message =
        field.params.get("message").and_then(|value| value.as_str())?;

    let threshold = if threshold_spec.starts_with("$.") {
        resolve_metadata_value(metadata, threshold_spec)
            .and_then(parse_biguint_from_value)
    } else {
        parse_biguint(threshold_spec)
    }?;

    if amount >= &threshold { Some(message.to_string()) } else { None }
}

fn format_enum(
    field: &EffectiveField,
    value: &Value,
    metadata: &Value,
) -> Result<String, Eip712Error> {
    let Some(reference) = field.params.get("$ref").and_then(|v| v.as_str())
    else {
        return Ok(format_raw(value));
    };

    let Some(enum_map) = resolve_metadata_value(metadata, reference) else {
        return Ok(format_raw(value));
    };

    if let Some(mapping) = enum_map.as_object()
        && let Some(text) = value_as_string(value)
        && let Some(label_value) = mapping.get(&text)
        && let Some(label) = label_value.as_str()
    {
        return Ok(label.to_string());
    }

    Ok(format_raw(value))
}

fn format_date(value: &Value) -> String {
    let Some(amount) = parse_biguint_from_value(value) else {
        return format_raw(value);
    };

    let Ok(seconds) = u64::try_from(amount) else {
        return format_raw(value);
    };
    let Ok(timestamp) = i64::try_from(seconds) else {
        return format_raw(value);
    };
    let Ok(datetime) = OffsetDateTime::from_unix_timestamp(timestamp) else {
        return format_raw(value);
    };
    let format = format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second] UTC"
    );
    datetime
        .to_offset(time::UtcOffset::UTC)
        .format(&format)
        .unwrap_or_else(|_| format_raw(value))
}

fn format_number(value: &Value) -> String {
    if let Some(number) = value.as_u64() {
        number.to_string()
    } else if let Some(text) = value_as_string(value) {
        text
    } else {
        format_raw(value)
    }
}

fn format_address(
    value: &Value,
    address_book: &HashMap<String, String>,
) -> String {
    let Some(address) = value_as_string(value) else {
        return format_raw(value);
    };

    let cleaned = address.trim();
    let bytes = match hex::decode(cleaned.trim_start_matches("0x")) {
        Ok(bytes) => bytes,
        Err(_) => return address,
    };
    if bytes.len() != 20 {
        return address;
    }
    let mut arr = [0u8; 20];
    arr.copy_from_slice(&bytes);
    let checksum = super::engine::to_checksum_address(&arr);
    let normalized = cleaned.to_ascii_lowercase();

    if let Some(label) = address_book.get(&normalized) {
        return label.clone();
    }

    checksum
}

fn format_raw(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        _ => value.to_string(),
    }
}

fn extract_chain_id(domain: &Value) -> Result<u64, Eip712Error> {
    let Some(chain_value) = domain.get("chainId") else {
        return Err(Eip712Error::TypedData(
            "typed data domain missing chainId".to_string(),
        ));
    };
    if let Some(number) = chain_value.as_u64() {
        Ok(number)
    } else if let Some(text) = chain_value.as_str() {
        if let Some(value) = parse_biguint(text) {
            value.try_into().map_err(|_| {
                Eip712Error::TypedData("chainId out of range".to_string())
            })
        } else {
            Err(Eip712Error::TypedData(
                "chainId is not a valid integer".to_string(),
            ))
        }
    } else {
        Err(Eip712Error::TypedData(
            "chainId must be a number or string".to_string(),
        ))
    }
}

fn extract_verifying_contract(domain: &Value) -> Result<String, Eip712Error> {
    let Some(value) = domain.get("verifyingContract").and_then(value_as_string)
    else {
        return Err(Eip712Error::TypedData(
            "typed data domain missing verifyingContract".to_string(),
        ));
    };
    Ok(value.to_ascii_lowercase())
}

fn get_value<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = root;
    let trimmed = path.trim();
    let trimmed = trimmed.strip_prefix("@.").unwrap_or(trimmed);
    if trimmed.is_empty() {
        return Some(current);
    }
    for segment in trimmed.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

fn parse_biguint_from_value(value: &Value) -> Option<BigUint> {
    match value {
        Value::String(text) => parse_biguint(text),
        Value::Number(number) => number.as_u64().map(BigUint::from),
        _ => None,
    }
}

fn extract_address_value(value: &Value) -> Option<String> {
    let text = value_as_string(value)?;
    if text.starts_with("0x") && text.len() == 42 {
        Some(text.to_ascii_lowercase())
    } else {
        None
    }
}

fn value_as_string(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}
