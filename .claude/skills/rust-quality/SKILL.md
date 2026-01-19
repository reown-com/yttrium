---
name: rust-quality
description: Writes high-quality Rust code following yttrium project standards. Use when working on Rust codebases, creating new modules, or reviewing Rust code quality.
---

# Rust Quality

## Goal
Write idiomatic, maintainable Rust code that follows the project's established patterns for error handling, type safety, cross-platform compatibility, and code organization.

## When to use
- Writing new Rust modules or functions
- Refactoring existing Rust code
- Adding cross-platform support (iOS, Android, WASM)
- Implementing error types
- Creating newtypes or domain types
- Writing tests

## When not to use
- Non-Rust codebases
- Simple documentation edits
- Configuration-only changes

## Default workflow
1. Read existing code in the module/area being modified
2. Follow established patterns for imports, types, errors
3. Write implementation with proper feature gates
4. Add inline tests in `#[cfg(test)] mod tests`
5. Run `just lint` to verify formatting and clippy

## Code Style

### Imports (block style, grouped)
```rust
use {
    crate::{
        module_a::TypeA,
        module_b::{TypeB, TypeC},
    },
    external_crate::{ExternalType, AnotherType},
    std::collections::HashMap,
};
```

### Error handling (thiserror + uniffi)
```rust
#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
pub enum MyError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found")]
    NotFound,

    #[error("Internal: {0}")]
    Internal(String),
}
```

### Newtypes (full derive chain)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct MyId(u64);

impl MyId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl From<u64> for MyId {
    fn from(val: u64) -> Self {
        Self::new(val)
    }
}

impl std::fmt::Display for MyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

### Structs with FFI support
```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct MyConfig {
    pub endpoint: String,
    pub timeout_ms: u64,
}
```

### Feature-gated modules
```rust
// In lib.rs
#[cfg(feature = "my_feature")]
pub mod my_module;

#[cfg(any(feature = "feature_a", feature = "feature_b"))]
pub mod shared_module;
```

### Async functions
```rust
pub async fn fetch_data(&self) -> eyre::Result<Data> {
    let response = self.client.get(&self.endpoint).await?;
    let data: Data = response.json().await?;
    Ok(data)
}
```

### Inline tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_id_creation() -> eyre::Result<()> {
        let id = MyId::new(42);
        eyre::ensure!(id.0 == 42, "id should be 42");
        Ok(())
    }

    #[tokio::test]
    async fn test_async_fetch() {
        let result = fetch_data().await;
        assert!(result.is_ok());
    }
}
```

### Impl block organization
```rust
impl MyClient {
    // 1. Constructor
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    // 2. Getters
    pub fn config(&self) -> &Config {
        &self.config
    }

    // 3. Preparation (side-effect free)
    pub fn prepare_request(&self, input: Input) -> PreparedRequest {
        // ...
    }

    // 4. Async operations
    pub async fn execute(&self, request: PreparedRequest) -> eyre::Result<Output> {
        // ...
    }
}
```

## Formatting rules
- Max line width: 80 characters
- Use `cargo +nightly fmt --all` for formatting
- Group imports with `imports_granularity = "One"`
- All clippy warnings are errors (`-D warnings`)

## Validation checklist
- [ ] Imports use block style and are grouped (crate, external, std)
- [ ] Errors derive `thiserror::Error` with descriptive messages
- [ ] FFI types have `#[cfg_attr(feature = "uniffi", ...)]`
- [ ] Newtypes have From/Into impls and Display
- [ ] Tests are inline in `#[cfg(test)] mod tests`
- [ ] No clippy warnings
- [ ] Line width under 80 characters
- [ ] Feature gates for platform-specific code

## Anti-patterns to avoid
- Stringly-typed APIs (use newtypes)
- `unwrap()` in library code (use `?` with eyre::Result)
- Deeply nested imports (flatten to specific types)
- Missing feature gates for platform code
- Tests in separate files when inline suffices
- Verbose comments (code should be self-documenting)

## Advanced Patterns

See [REFERENCE.md](./REFERENCE.md) for:
- Custom serde modules (duration_millis, etc.)
- UniFFI custom type mappings
- Platform-aware async spawning (native vs WASM)
- Async trait with WASM support (`?Send`)
- Thread-safe caching with `Arc<RwLock<_>>`
- Serde enum patterns (tagged unions, rename_all_fields)
- Feature flag organization in Cargo.toml
- Constants module pattern

## Examples

### Example 1: New error type
Input: "Add an error type for payment failures"
```rust
#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
pub enum PaymentError {
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u64, available: u64 },

    #[error("Invalid recipient: {0}")]
    InvalidRecipient(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),
}
```

### Example 2: Cross-platform struct
Input: "Create a transaction request type for all platforms"
```rust
use {
    crate::chain::ChainId,
    alloy::primitives::{Address, U256},
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct TransactionRequest {
    pub chain_id: ChainId,
    pub to: Address,
    pub value: U256,
    pub data: Vec<u8>,
}

impl TransactionRequest {
    pub fn new(chain_id: ChainId, to: Address, value: U256, data: Vec<u8>) -> Self {
        Self { chain_id, to, value, data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_request_creation() {
        let req = TransactionRequest::new(
            ChainId::new_eip155(1),
            Address::ZERO,
            U256::ZERO,
            vec![],
        );
        assert_eq!(req.chain_id, ChainId::new_eip155(1));
    }
}
```
