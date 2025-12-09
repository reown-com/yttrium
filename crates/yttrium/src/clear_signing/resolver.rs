//! Descriptor lookup and token metadata resolution entry points for clear signing.

use {
    super::{
        descriptor::{
            build_descriptor, decode_arguments, determine_token_key,
            native_token_key, resolve_effective_field, Descriptor,
            DescriptorError, TokenLookupError, TokenLookupKey,
        },
        token_registry::{lookup_token_by_caip19, TokenMeta},
    },
    serde::Deserialize,
    serde_json::Value,
    std::{
        collections::{HashMap, HashSet},
        sync::OnceLock,
    },
    thiserror::Error,
};

pub struct ResolvedDescriptor<'a> {
    pub descriptor_json: &'a str,
    pub abi_json: Option<&'a str>,
    pub includes: Vec<&'a str>,
}

pub struct ResolvedCall<'a> {
    pub descriptor: ResolvedDescriptor<'a>,
    pub token_metadata: HashMap<TokenLookupKey, TokenMeta>,
    pub address_book: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("descriptor not found for {0}")]
    NotFound(String),
    #[error("invalid index entry for {path}")]
    InvalidIndexEntry { path: String },
    #[error("include not found: {0}")]
    IncludeNotFound(String),
    #[error("descriptor parse error: {0}")]
    DescriptorParse(String),
}

#[derive(Debug, Deserialize)]
struct IndexEntry {
    descriptor: String,
    #[serde(default)]
    abi: Option<String>,
}

type IndexMap = HashMap<String, IndexEntry>;

const INDEX_JSON: &str = include_str!("assets/index.json");

const DESCRIPTOR_ERC20_USDT: &str =
    include_str!("assets/descriptors/erc20_usdt.json");
const DESCRIPTOR_UNISWAP_V3_ROUTER_V1: &str =
    include_str!("assets/descriptors/uniswap_v3_router_v1.json");
const DESCRIPTOR_WETH9: &str = include_str!("assets/descriptors/weth9.json");
const DESCRIPTOR_AGGREGATION_ROUTER_V4: &str =
    include_str!("assets/descriptors/aggregation_router_v4.json");
const DESCRIPTOR_1INCH_AGG_ROUTER_V3: &str =
    include_str!("assets/descriptors/1inch/calldata-AggregationRouterV3.json");
const DESCRIPTOR_1INCH_AGG_ROUTER_V4_ETH: &str = include_str!(
    "assets/descriptors/1inch/calldata-AggregationRouterV4-eth.json"
);
const DESCRIPTOR_1INCH_AGG_ROUTER_V4: &str =
    include_str!("assets/descriptors/1inch/calldata-AggregationRouterV4.json");
const DESCRIPTOR_1INCH_AGG_ROUTER_V5: &str =
    include_str!("assets/descriptors/1inch/calldata-AggregationRouterV5.json");
const DESCRIPTOR_1INCH_AGG_ROUTER_V6: &str =
    include_str!("assets/descriptors/1inch/calldata-AggregationRouterV6.json");
const DESCRIPTOR_1INCH_AGG_ROUTER_V6_ZKSYNC: &str = include_str!(
    "assets/descriptors/1inch/calldata-AggregationRouterV6-zksync.json"
);
const DESCRIPTOR_1INCH_NATIVE_ORDER_FACTORY: &str =
    include_str!("assets/descriptors/1inch/calldata-NativeOrderFactory.json");
const DESCRIPTOR_AAVE_LPV2: &str =
    include_str!("assets/descriptors/aave/calldata-lpv2.json");
const DESCRIPTOR_AAVE_LPV3: &str =
    include_str!("assets/descriptors/aave/calldata-lpv3.json");
const DESCRIPTOR_AAVE_WETH_GATEWAY_V3: &str =
    include_str!("assets/descriptors/aave/calldata-WrappedTokenGatewayV3.json");
const DESCRIPTOR_WALLETC_W_STAKEWEIGHT: &str =
    include_str!("assets/descriptors/walletconnect/calldata-stakeweight.json");
const INCLUDE_COMMON_TEST_ROUTER: &str =
    include_str!("assets/descriptors/common-test-router.json");
const INCLUDE_1INCH_COMMON_V4: &str =
    include_str!("assets/descriptors/1inch/common-AggregationRouterV4.json");
const INCLUDE_1INCH_COMMON_V6: &str =
    include_str!("assets/descriptors/1inch/common-AggregationRouterV6.json");
const INCLUDE_UNISWAP_COMMON_EIP712: &str =
    include_str!("assets/descriptors/uniswap/uniswap-common-eip712.json");

const ABI_ERC20: &str = include_str!("assets/abis/erc20.json");
const ABI_UNISWAP_V3_ROUTER_V1: &str =
    include_str!("assets/abis/uniswap_v3_router_v1.json");
const ABI_WETH9: &str = include_str!("assets/abis/weth9.json");

static INDEX: OnceLock<IndexMap> = OnceLock::new();
const TYPED_INDEX_JSON: &str = include_str!("assets/index_eip712.json");
const TYPED_DESCRIPTOR_1INCH_LIMIT_ORDER: &str =
    include_str!("assets/descriptors/1inch/eip712-1inch-limit-order.json");
const TYPED_DESCRIPTOR_1INCH_AGG_ROUTER_V6: &str =
    include_str!("assets/descriptors/1inch/eip712-AggregationRouterV6.json");
const TYPED_DESCRIPTOR_UNISWAP_PERMIT2: &str =
    include_str!("assets/descriptors/uniswap/eip712-uniswap-permit2.json");
type TypedIndexMap = HashMap<String, String>;
static TYPED_INDEX: OnceLock<TypedIndexMap> = OnceLock::new();
const ADDRESS_BOOK_JSON: &str = include_str!("assets/address_book.json");
type ChainAddressBook = HashMap<String, HashMap<String, String>>;
static ADDRESS_BOOK: OnceLock<ChainAddressBook> = OnceLock::new();

fn index() -> &'static IndexMap {
    INDEX.get_or_init(|| {
        serde_json::from_str(INDEX_JSON)
            .expect("clear signing index JSON must be valid")
    })
}

fn typed_index() -> &'static TypedIndexMap {
    TYPED_INDEX.get_or_init(|| {
        serde_json::from_str(TYPED_INDEX_JSON)
            .expect("clear signing typed index JSON must be valid")
    })
}

/// Resolves a descriptor bundle and associated assets for the given chain and address.
pub fn resolve(
    chain_id: u64,
    to: &str,
) -> Result<ResolvedDescriptor<'static>, ResolverError> {
    eprintln!("[resolver] resolve request chain_id={} to={}", chain_id, to);
    let key = format!("eip155:{}:{}", chain_id, normalize_address(to));
    eprintln!("[resolver] lookup key {}", key);
    let entry = index()
        .get(&key)
        .ok_or_else(|| ResolverError::NotFound(key.clone()))?;

    let descriptor =
        descriptor_content(&entry.descriptor).ok_or_else(|| {
            ResolverError::InvalidIndexEntry { path: entry.descriptor.clone() }
        })?;
    eprintln!(
        "[resolver] descriptor path {} length {}",
        entry.descriptor,
        descriptor.len()
    );

    let abi = match entry.abi.as_deref() {
        Some(path) => {
            eprintln!("[resolver] abi path {}", path);
            Some(abi_content(path).ok_or_else(|| {
                ResolverError::InvalidIndexEntry { path: path.to_string() }
            })?)
        }
        None => None,
    };

    let includes = extract_includes(descriptor)?;
    if !includes.is_empty() {
        eprintln!("[resolver] includes count {}", includes.len());
    }

    Ok(ResolvedDescriptor {
        descriptor_json: descriptor,
        abi_json: abi,
        includes,
    })
}

/// Resolves a descriptor and fetches token metadata required for rendering a call preview.
pub fn resolve_call(
    chain_id: u64,
    to: &str,
    calldata: &[u8],
    value: Option<&[u8]>,
) -> Result<ResolvedCall<'static>, ResolverError> {
    let resolved = resolve(chain_id, to)?;
    let descriptor_struct =
        build_descriptor(&resolved).map_err(map_descriptor_error)?;

    let selector = selector_from_calldata(calldata)?;

    let functions = descriptor_struct
        .context
        .contract
        .function_descriptors()
        .map_err(map_descriptor_error)?;

    let mut token_metadata = HashMap::new();
    if let Some(function) =
        functions.iter().find(|func| func.selector == selector)
    {
        let decoded = decode_arguments(function, calldata)
            .and_then(|decoded| decoded.with_value(value))
            .map_err(map_descriptor_error)?;

        let mut keys: HashSet<TokenLookupKey> = HashSet::new();
        if let Some(format_def) = descriptor_struct
            .display
            .format_map()
            .get(&function.typed_signature)
        {
            let mut warnings = Vec::new();
            for field in &format_def.fields {
                let Some(effective) = resolve_effective_field(
                    field,
                    descriptor_struct.display.definitions(),
                    &mut warnings,
                ) else {
                    continue;
                };

                match effective.format.as_deref() {
                    Some("tokenAmount") => {
                        let key = determine_token_key(
                            &effective, &decoded, chain_id, to,
                        )
                        .map_err(map_token_lookup_error)?;
                        keys.insert(key);
                    }
                    Some("amount") => {
                        let key = native_token_key(chain_id)
                            .map_err(map_token_lookup_error)?;
                        keys.insert(key);
                    }
                    _ => {}
                }
            }
        }

        for key in keys {
            let meta =
                lookup_token_by_caip19(key.as_str()).ok_or_else(|| {
                    ResolverError::DescriptorParse(format!(
                        "token registry missing entry for {:?}",
                        key
                    ))
                })?;
            token_metadata.insert(key, meta);
        }
    }

    let mut address_book = descriptor_address_book(&descriptor_struct);
    merge_registry_entries(&mut address_book, chain_id);

    Ok(ResolvedCall { descriptor: resolved, token_metadata, address_book })
}

fn descriptor_content(path: &str) -> Option<&'static str> {
    match path {
        "descriptors/erc20_usdt.json" => Some(DESCRIPTOR_ERC20_USDT),
        "descriptors/erc20_usdc.json" => {
            Some(include_str!("assets/descriptors/erc20_usdc.json"))
        }
        "descriptors/uniswap_v3_router_v1.json" => {
            Some(DESCRIPTOR_UNISWAP_V3_ROUTER_V1)
        }
        "descriptors/weth9.json" => Some(DESCRIPTOR_WETH9),
        "descriptors/aggregation_router_v4.json" => {
            Some(DESCRIPTOR_AGGREGATION_ROUTER_V4)
        }
        "descriptors/1inch/calldata-AggregationRouterV3.json" => {
            Some(DESCRIPTOR_1INCH_AGG_ROUTER_V3)
        }
        "descriptors/1inch/calldata-AggregationRouterV4-eth.json" => {
            Some(DESCRIPTOR_1INCH_AGG_ROUTER_V4_ETH)
        }
        "descriptors/1inch/calldata-AggregationRouterV4.json" => {
            Some(DESCRIPTOR_1INCH_AGG_ROUTER_V4)
        }
        "descriptors/1inch/calldata-AggregationRouterV5.json" => {
            Some(DESCRIPTOR_1INCH_AGG_ROUTER_V5)
        }
        "descriptors/1inch/calldata-AggregationRouterV6.json" => {
            Some(DESCRIPTOR_1INCH_AGG_ROUTER_V6)
        }
        "descriptors/1inch/calldata-AggregationRouterV6-zksync.json" => {
            Some(DESCRIPTOR_1INCH_AGG_ROUTER_V6_ZKSYNC)
        }
        "descriptors/1inch/calldata-NativeOrderFactory.json" => {
            Some(DESCRIPTOR_1INCH_NATIVE_ORDER_FACTORY)
        }
        "descriptors/aave/calldata-lpv2.json" => Some(DESCRIPTOR_AAVE_LPV2),
        "descriptors/aave/calldata-lpv3.json" => Some(DESCRIPTOR_AAVE_LPV3),
        "descriptors/aave/calldata-WrappedTokenGatewayV3.json" => {
            Some(DESCRIPTOR_AAVE_WETH_GATEWAY_V3)
        }
        "descriptors/walletconnect/calldata-stakeweight.json" => {
            Some(DESCRIPTOR_WALLETC_W_STAKEWEIGHT)
        }
        _ => None,
    }
}

fn abi_content(path: &str) -> Option<&'static str> {
    match path {
        "abis/erc20.json" => Some(ABI_ERC20),
        "abis/uniswap_v3_router_v1.json" => Some(ABI_UNISWAP_V3_ROUTER_V1),
        "abis/weth9.json" => Some(ABI_WETH9),
        "abis/1inch/aggregation_router_v3.json" => {
            Some(include_str!("assets/abis/1inch/aggregation_router_v3.json"))
        }
        "abis/1inch/aggregation_router_v4.json" => {
            Some(include_str!("assets/abis/1inch/aggregation_router_v4.json"))
        }
        "abis/1inch/aggregation_router_v5.json" => {
            Some(include_str!("assets/abis/1inch/aggregation_router_v5.json"))
        }
        "abis/1inch/aggregation_router_v6.json" => {
            Some(include_str!("assets/abis/1inch/aggregation_router_v6.json"))
        }
        "abis/1inch/native_order_factory.json" => {
            Some(include_str!("assets/abis/1inch/native_order_factory.json"))
        }
        "abis/aave/lpv2.json" => {
            Some(include_str!("assets/abis/aave/lpv2.json"))
        }
        "abis/aave/lpv3.json" => {
            Some(include_str!("assets/abis/aave/lpv3.json"))
        }
        "abis/aave/weth_gateway_v3.json" => {
            Some(include_str!("assets/abis/aave/weth_gateway_v3.json"))
        }
        _ => None,
    }
}

fn map_descriptor_error(err: DescriptorError) -> ResolverError {
    match err {
        DescriptorError::Parse(message) => {
            ResolverError::DescriptorParse(message)
        }
        DescriptorError::Calldata(message) => {
            ResolverError::DescriptorParse(message)
        }
    }
}

fn map_token_lookup_error(err: TokenLookupError) -> ResolverError {
    ResolverError::DescriptorParse(err.to_string())
}

fn selector_from_calldata(calldata: &[u8]) -> Result<[u8; 4], ResolverError> {
    if calldata.len() < 4 {
        return Err(ResolverError::DescriptorParse(
            "calldata must be at least 4 bytes".to_string(),
        ));
    }
    let mut selector = [0u8; 4];
    selector.copy_from_slice(&calldata[0..4]);
    Ok(selector)
}

fn normalize_address(address: &str) -> String {
    address.trim().to_ascii_lowercase()
}

pub struct ResolvedTypedDescriptor<'a> {
    pub descriptor_json: &'a str,
    pub includes: Vec<&'a str>,
    pub address_book: HashMap<String, String>,
}

pub fn resolve_typed(
    chain_id: u64,
    verifying_contract: &str,
) -> Result<ResolvedTypedDescriptor<'static>, ResolverError> {
    let key = format!(
        "eip155:{}:{}",
        chain_id,
        normalize_address(verifying_contract)
    );
    let path = typed_index()
        .get(&key)
        .ok_or_else(|| ResolverError::NotFound(key.clone()))?;

    let descriptor = typed_descriptor_content(path).ok_or_else(|| {
        ResolverError::InvalidIndexEntry { path: path.clone() }
    })?;

    let includes = extract_includes(descriptor)?;

    let address_book = build_typed_address_book(
        descriptor,
        &includes,
        chain_id,
        verifying_contract,
    )?;

    Ok(ResolvedTypedDescriptor {
        descriptor_json: descriptor,
        includes,
        address_book,
    })
}

fn typed_descriptor_content(path: &str) -> Option<&'static str> {
    match path {
        "descriptors/1inch/eip712-1inch-limit-order.json" => {
            Some(TYPED_DESCRIPTOR_1INCH_LIMIT_ORDER)
        }
        "descriptors/1inch/eip712-AggregationRouterV6.json" => {
            Some(TYPED_DESCRIPTOR_1INCH_AGG_ROUTER_V6)
        }
        "descriptors/uniswap/eip712-uniswap-permit2.json" => {
            Some(TYPED_DESCRIPTOR_UNISWAP_PERMIT2)
        }
        _ => None,
    }
}

fn extract_includes(
    descriptor_json: &str,
) -> Result<Vec<&'static str>, ResolverError> {
    let value: Value = serde_json::from_str(descriptor_json)
        .map_err(|err| ResolverError::DescriptorParse(err.to_string()))?;

    let Some(includes_value) = value.get("includes") else {
        return Ok(Vec::new());
    };

    let mut includes = Vec::new();
    match includes_value {
        Value::String(name) => {
            includes.push(
                include_content(name).ok_or_else(|| {
                    ResolverError::IncludeNotFound(name.clone())
                })?,
            );
        }
        Value::Array(items) => {
            for item in items {
                let name = item.as_str().ok_or_else(|| {
                    ResolverError::DescriptorParse(
                        "includes entries must be strings".to_string(),
                    )
                })?;
                includes.push(include_content(name).ok_or_else(|| {
                    ResolverError::IncludeNotFound(name.to_string())
                })?);
            }
        }
        _ => {
            return Err(ResolverError::DescriptorParse(
                "\"includes\" must be string or array".to_string(),
            ));
        }
    };

    Ok(includes)
}

fn include_content(name: &str) -> Option<&'static str> {
    match name {
        "common-test-router.json" => Some(INCLUDE_COMMON_TEST_ROUTER),
        "common-AggregationRouterV4.json" => Some(INCLUDE_1INCH_COMMON_V4),
        "common-AggregationRouterV6.json" => Some(INCLUDE_1INCH_COMMON_V6),
        "uniswap-common-eip712.json" => Some(INCLUDE_UNISWAP_COMMON_EIP712),
        _ => None,
    }
}

fn descriptor_address_book(descriptor: &Descriptor) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(label) = descriptor_friendly_label(descriptor) {
        for deployment in &descriptor.context.contract.deployments {
            map.insert(normalize_address(&deployment.address), label.clone());
        }
    }
    merge_address_book_entries(
        &mut map,
        descriptor.metadata.get("addressBook"),
    );
    map
}

fn descriptor_friendly_label(descriptor: &Descriptor) -> Option<String> {
    metadata_label(&descriptor.metadata)
        .or_else(|| descriptor.context.id.clone())
}

fn metadata_label(metadata: &serde_json::Value) -> Option<String> {
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

    None
}

fn merge_address_book_entries(
    map: &mut HashMap<String, String>,
    value: Option<&serde_json::Value>,
) {
    let Some(value) = value else { return };
    let Some(entries) = value.as_object() else { return };

    for (key, label_value) in entries {
        match label_value {
            serde_json::Value::String(label) => {
                map.entry(normalize_address(key))
                    .or_insert_with(|| label.to_string());
            }
            serde_json::Value::Object(nested) => {
                for (inner_key, inner_label_value) in nested {
                    if let Some(inner_label) = inner_label_value.as_str() {
                        map.entry(normalize_address(inner_key))
                            .or_insert_with(|| inner_label.to_string());
                    }
                }
            }
            _ => continue,
        }
    }
}

fn merge_registry_entries(map: &mut HashMap<String, String>, chain_id: u64) {
    let key = format!("eip155:{chain_id}").to_ascii_lowercase();
    if let Some(entries) = registry_address_book().get(&key) {
        for (address, label) in entries {
            map.entry(address.clone()).or_insert_with(|| label.clone());
        }
    }
}

fn registry_address_book() -> &'static ChainAddressBook {
    ADDRESS_BOOK.get_or_init(load_address_book_registry)
}

fn load_address_book_registry() -> ChainAddressBook {
    let mut map = HashMap::new();
    if let Ok(value) =
        serde_json::from_str::<serde_json::Value>(ADDRESS_BOOK_JSON)
    {
        if let Some(entries) = value.as_object() {
            for (chain_key, addresses_value) in entries {
                let Some(addresses) = addresses_value.as_object() else {
                    continue;
                };
                let normalized_chain = chain_key.trim().to_ascii_lowercase();
                let mut chain_map = HashMap::new();
                for (address, label_value) in addresses {
                    if let Some(label) = label_value.as_str() {
                        chain_map.insert(
                            normalize_address(address),
                            label.to_string(),
                        );
                    }
                }
                map.insert(normalized_chain, chain_map);
            }
        }
    }
    map
}

fn build_typed_address_book(
    descriptor_json: &str,
    includes: &[&str],
    chain_id: u64,
    verifying_contract: &str,
) -> Result<HashMap<String, String>, ResolverError> {
    let descriptor_value = merged_descriptor_value(descriptor_json, includes)?;
    let mut map = HashMap::new();

    if let Some(metadata) = descriptor_value.get("metadata") {
        if let Some(label) = metadata_label(metadata) {
            if let Some(deployments) = descriptor_value
                .get("context")
                .and_then(|ctx| ctx.get("eip712"))
                .and_then(|value| value.get("deployments"))
                .and_then(|value| value.as_array())
            {
                for deployment in deployments {
                    let Some(address) = deployment
                        .get("address")
                        .and_then(|value| value.as_str())
                    else {
                        continue;
                    };
                    map.insert(normalize_address(address), label.clone());
                }
            }

            map.entry(normalize_address(verifying_contract))
                .or_insert_with(|| label.clone());
        }

        merge_address_book_entries(&mut map, metadata.get("addressBook"));
    }

    merge_registry_entries(&mut map, chain_id);

    Ok(map)
}

pub(crate) fn merged_descriptor_value(
    descriptor_json: &str,
    includes: &[&str],
) -> Result<Value, ResolverError> {
    let mut descriptor_value: Value = serde_json::from_str(descriptor_json)
        .map_err(|err| ResolverError::DescriptorParse(err.to_string()))?;

    for include_json in includes {
        let include_value: Value = serde_json::from_str(include_json)
            .map_err(|err| ResolverError::DescriptorParse(err.to_string()))?;
        merge_include_value(&mut descriptor_value, include_value);
    }

    if let Some(object) = descriptor_value.as_object_mut() {
        object.remove("includes");
    }

    Ok(descriptor_value)
}

fn merge_include_value(target: &mut Value, include: Value) {
    if let (Value::Object(target_map), Value::Object(include_map)) =
        (target, include)
    {
        for (key, value) in include_map {
            match target_map.get_mut(&key) {
                Some(existing) => {
                    if existing.is_object() && value.is_object() {
                        merge_include_value(existing, value);
                    }
                }
                None => {
                    target_map.insert(key, value);
                }
            }
        }
    }
}
