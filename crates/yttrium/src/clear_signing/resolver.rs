use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

pub struct ResolvedDescriptor<'a> {
    pub descriptor_json: &'a str,
    pub abi_json: Option<&'a str>,
    pub includes: Vec<&'a str>,
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
const INCLUDE_COMMON_TEST_ROUTER: &str =
    include_str!("assets/descriptors/common-test-router.json");
const INCLUDE_1INCH_COMMON_V4: &str =
    include_str!("assets/descriptors/1inch/common-AggregationRouterV4.json");
const INCLUDE_1INCH_COMMON_V6: &str =
    include_str!("assets/descriptors/1inch/common-AggregationRouterV6.json");

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
type TypedIndexMap = HashMap<String, String>;
static TYPED_INDEX: OnceLock<TypedIndexMap> = OnceLock::new();

fn load_index() -> IndexMap {
    serde_json::from_str(INDEX_JSON)
        .expect("clear signing index JSON must be valid")
}

fn index() -> &'static IndexMap {
    INDEX.get_or_init(load_index)
}

fn load_typed_index() -> TypedIndexMap {
    serde_json::from_str(TYPED_INDEX_JSON)
        .expect("clear signing typed index JSON must be valid")
}

fn typed_index() -> &'static TypedIndexMap {
    TYPED_INDEX.get_or_init(load_typed_index)
}

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

fn descriptor_content(path: &str) -> Option<&'static str> {
    match path {
        "descriptors/erc20_usdt.json" => Some(DESCRIPTOR_ERC20_USDT),
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

fn normalize_address(address: &str) -> String {
    address.trim().to_ascii_lowercase()
}

pub struct ResolvedTypedDescriptor<'a> {
    pub descriptor_json: &'a str,
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

    Ok(ResolvedTypedDescriptor { descriptor_json: descriptor })
}

fn typed_descriptor_content(path: &str) -> Option<&'static str> {
    match path {
        "descriptors/1inch/eip712-1inch-limit-order.json" => {
            Some(TYPED_DESCRIPTOR_1INCH_LIMIT_ORDER)
        }
        "descriptors/1inch/eip712-AggregationRouterV6.json" => {
            Some(TYPED_DESCRIPTOR_1INCH_AGG_ROUTER_V6)
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
            ))
        }
    };

    Ok(includes)
}

fn include_content(name: &str) -> Option<&'static str> {
    match name {
        "common-test-router.json" => Some(INCLUDE_COMMON_TEST_ROUTER),
        "common-AggregationRouterV4.json" => Some(INCLUDE_1INCH_COMMON_V4),
        "common-AggregationRouterV6.json" => Some(INCLUDE_1INCH_COMMON_V6),
        _ => None,
    }
}
