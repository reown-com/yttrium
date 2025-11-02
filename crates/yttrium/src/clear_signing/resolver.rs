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
const INCLUDE_COMMON_TEST_ROUTER: &str =
    include_str!("assets/descriptors/common-test-router.json");

const ABI_ERC20: &str = include_str!("assets/abis/erc20.json");
const ABI_UNISWAP_V3_ROUTER_V1: &str =
    include_str!("assets/abis/uniswap_v3_router_v1.json");
const ABI_WETH9: &str = include_str!("assets/abis/weth9.json");

static INDEX: OnceLock<IndexMap> = OnceLock::new();

fn load_index() -> IndexMap {
    serde_json::from_str(INDEX_JSON)
        .expect("clear signing index JSON must be valid")
}

fn index() -> &'static IndexMap {
    INDEX.get_or_init(load_index)
}

/// Resolves the descriptor and ABI (if present) for the given target.
pub fn resolve(
    chain_id: u64,
    to: &str,
) -> Result<ResolvedDescriptor<'static>, ResolverError> {
    let key = format!("eip155:{}:{}", chain_id, normalize_address(to));
    let entry = index()
        .get(&key)
        .ok_or_else(|| ResolverError::NotFound(key.clone()))?;

    let descriptor =
        descriptor_content(&entry.descriptor).ok_or_else(|| {
            ResolverError::InvalidIndexEntry { path: entry.descriptor.clone() }
        })?;

    let abi = match entry.abi.as_deref() {
        Some(path) => Some(abi_content(path).ok_or_else(|| {
            ResolverError::InvalidIndexEntry { path: path.to_string() }
        })?),
        None => None,
    };

    let includes = extract_includes(descriptor)?;

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
        _ => None,
    }
}

fn abi_content(path: &str) -> Option<&'static str> {
    match path {
        "abis/erc20.json" => Some(ABI_ERC20),
        "abis/uniswap_v3_router_v1.json" => Some(ABI_UNISWAP_V3_ROUTER_V1),
        "abis/weth9.json" => Some(ABI_WETH9),
        _ => None,
    }
}

fn normalize_address(address: &str) -> String {
    address.trim().to_ascii_lowercase()
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
        _ => None,
    }
}
