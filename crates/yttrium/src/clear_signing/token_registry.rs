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

#[derive(Debug, Deserialize)]
struct TokenRegistryEntry {
    symbol: String,
    decimals: u8,
    name: String,
}

static TOKEN_REGISTRY: OnceLock<HashMap<String, TokenMeta>> = OnceLock::new();

pub fn lookup_token_by_caip19(caip19: &str) -> Option<TokenMeta> {
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

