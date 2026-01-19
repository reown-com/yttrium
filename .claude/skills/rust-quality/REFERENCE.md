# Rust Quality Reference

Advanced patterns and detailed examples for the rust-quality skill.

## Custom Serde Modules

For domain-specific serialization:

```rust
pub mod duration_millis {
    use {
        serde::{de, ser, Deserialize},
        std::time::Duration,
    };

    pub fn serialize<S>(dt: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_u128(dt.as_millis())
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        u64::deserialize(d).map(Duration::from_millis)
    }
}

// Usage:
#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(with = "duration_millis")]
    pub timeout: Duration,
}
```

## UniFFI Custom Type Mappings

For types that need custom FFI conversion:

```rust
uniffi::custom_type!(Address, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(ChainId, u64, {
    try_lift: |val| Ok(ChainId::new_eip155(val)),
    lower: |obj| obj.eip155_chain_id(),
});
```

## Platform-Aware Async Spawning

```rust
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<F>(future: F)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(future);
    } else {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime");
            rt.block_on(future);
        });
    }
}

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}
```

## Async Trait with WASM Support

```rust
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
pub trait DataProvider {
    async fn fetch(&self, id: &str) -> eyre::Result<Data>;
}
```

## Thread-Safe Caching Pattern

```rust
use {
    std::{collections::HashMap, sync::Arc},
    tokio::sync::RwLock,
};

pub struct ProviderPool {
    providers: Arc<RwLock<HashMap<String, Arc<Provider>>>>,
}

impl ProviderPool {
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_or_create(&self, endpoint: &str) -> Arc<Provider> {
        // Check read lock first
        {
            let cache = self.providers.read().await;
            if let Some(provider) = cache.get(endpoint) {
                return Arc::clone(provider);
            }
        }

        // Upgrade to write lock
        let mut cache = self.providers.write().await;
        // Double-check after acquiring write lock
        if let Some(provider) = cache.get(endpoint) {
            return Arc::clone(provider);
        }

        let provider = Arc::new(Provider::new(endpoint));
        cache.insert(endpoint.to_string(), Arc::clone(&provider));
        provider
    }
}
```

## Serde Enum Patterns

### Tagged unions (for JSON interop)
```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Action {
    #[serde(rename = "transfer")]
    Transfer { to: Address, amount: U256 },
    #[serde(rename = "approve")]
    Approve { spender: Address, amount: U256 },
}
```

### With rename_all_fields
```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Response {
    Success { txHash: String, blockNumber: u64 },
    Error { errorCode: u32, errorMessage: String },
}
```

## Feature Flag Patterns

### Cargo.toml structure
```toml
[features]
default = ["eip155", "erc6492_client"]

# Platform targets
all_platforms = ["android", "ios", "wasm"]
ios = ["uniffi"]
android = ["uniffi", "dep:jni"]
wasm = ["dep:wasm-bindgen", "dep:tsify-next"]

# Client features (additive)
all_clients = ["client_a", "client_b", "client_c"]
client_a = []
client_b = ["client_a"]  # client_b depends on client_a
client_c = ["dep:special-crate"]

# Aggregate
full = ["all_platforms", "all_clients"]
```

### Conditional compilation
```rust
// Module level
#[cfg(feature = "client_a")]
pub mod client_a;

// Item level
#[cfg(feature = "uniffi")]
impl MyType {
    pub fn for_ffi(&self) -> String {
        self.to_string()
    }
}

// Expression level
let client = {
    #[cfg(feature = "client_a")]
    { ClientA::new() }
    #[cfg(not(feature = "client_a"))]
    { compile_error!("client_a feature required") }
};
```

## Test Helpers Module

```rust
// src/test_helpers/mod.rs
#[cfg(test)]
pub mod mock_provider;

#[cfg(test)]
pub mod fixtures;

// Usage in tests
#[cfg(test)]
mod tests {
    use crate::test_helpers::{fixtures, mock_provider};

    #[test]
    fn test_with_mock() {
        let provider = mock_provider::create();
        let data = fixtures::sample_transaction();
        // ...
    }
}
```

## Constants Module Pattern

```rust
pub mod constants {
    pub mod methods {
        pub const SEND: &str = "eth_sendTransaction";
        pub const CALL: &str = "eth_call";
    }

    pub mod headers {
        pub const API_KEY: &str = "X-Api-Key";
        pub const PROJECT_ID: &str = "X-Project-Id";
    }

    pub mod defaults {
        pub const TIMEOUT_MS: u64 = 30_000;
        pub const MAX_RETRIES: u32 = 3;
    }
}
```
