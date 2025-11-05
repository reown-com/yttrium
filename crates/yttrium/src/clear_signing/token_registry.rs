use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

const TOKEN_REGISTRY_JSON: &str = include_str!("assets/tokens-min.json");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenMeta {
    pub symbol: String,
    pub decimals: u8,
    pub name: String,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TokenResolver;

pub const TOKEN_RESOLVER: TokenResolver = TokenResolver;

impl TokenResolver {
    pub fn lookup_erc20_token(
        self,
        chain_id: u64,
        address: &str,
    ) -> Option<TokenMeta> {
        lookup_erc20_token(chain_id, address)
    }

    pub fn lookup_native_token(self, chain_id: u64) -> Option<TokenMeta> {
        lookup_native_token(chain_id)
    }

    pub fn lookup_by_caip19(self, caip19: &str) -> Option<TokenMeta> {
        lookup_token_by_caip19(caip19)
    }
}

#[derive(Debug, Deserialize)]
struct TokenRegistryEntry {
    symbol: String,
    decimals: u8,
    name: String,
}

static TOKEN_REGISTRY: OnceLock<HashMap<String, TokenMeta>> = OnceLock::new();

fn lookup_erc20_token(chain_id: u64, address: &str) -> Option<TokenMeta> {
    let key = format!("eip155:{}/erc20:{}", chain_id, normalize(address));
    lookup_token_by_caip19(&key)
}

fn lookup_native_token(chain_id: u64) -> Option<TokenMeta> {
    let slip44 = native_slip44_code(chain_id)?;
    let key = format!("eip155:{}/slip44:{}", chain_id, slip44);
    lookup_token_by_caip19(&key)
}

fn lookup_token_by_caip19(caip19: &str) -> Option<TokenMeta> {
    let key = normalize(caip19);
    TOKEN_REGISTRY.get_or_init(load_registry).get(&key).cloned()
}

fn load_registry() -> HashMap<String, TokenMeta> {
    let raw: HashMap<String, TokenRegistryEntry> =
        serde_json::from_str(TOKEN_REGISTRY_JSON)
            .expect("token registry JSON should be valid");

    raw.into_iter()
        .map(|(key, entry)| {
            (
                key.to_ascii_lowercase(),
                TokenMeta {
                    symbol: entry.symbol,
                    decimals: entry.decimals,
                    name: entry.name,
                },
            )
        })
        .collect()
}

fn normalize(input: &str) -> String {
    input.trim().to_ascii_lowercase()
}

fn native_slip44_code(chain_id: u64) -> Option<u32> {
    match chain_id {
        1 | 10 | 42161 | 8453 => Some(60),
        _ => None,
    }
}
