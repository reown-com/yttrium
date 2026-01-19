# AGENTS.md

Guidance for AI agents working on this repository. Currently focused on the **pay module**.

## Build & Development Commands

```bash
# Development workflow
just devloop              # Regular dev loop: lint, test, format
just lint                 # cargo +nightly fmt --all + clippy
just test                 # Run all tests with --features=full

# Pay module tests
just test-pay-e2e         # Pay e2e tests (requires env vars)
cargo test --features=full --lib --bins pay

# Platform builds
just swift                # Build XCFramework and Swift bindings
just kotlin               # Build Android Kotlin bindings (NDK 26+)

# Single test
cargo test --features=full --lib --bins <test_name>
```

## Coding Standards

Reference `.claude/skills/rust-quality/` for detailed patterns. Key points:

- **Linting:** `just lint` runs `cargo +nightly fmt --all` + clippy with `-D warnings`
- **Line width:** 80 characters max
- **Imports:** Block style, grouped by crate/external/std
- **Errors:** Use `thiserror` with `#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]`
- **FFI types:** Add `#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]`
- **Tests:** Inline `#[cfg(test)] mod tests` blocks
- **Results:** Use `eyre::Result<T>` for error propagation, avoid `unwrap()`
- **Conciseness:** Avoid extra variable bindings, minimize comments

## Environment Variables

```bash
REOWN_PROJECT_ID          # Required for pay e2e tests
PIMLICO_API_KEY           # Pimlico tests (if needed)
ANDROID_NDK_HOME          # Android/Kotlin builds
```

## Pay Module Structure

```
crates/
├── yttrium/src/pay/
│   ├── mod.rs              # Main WalletConnectPay client (~1300 lines)
│   ├── json.rs             # WalletConnectPayJson - JSON wrapper for UniFFI
│   ├── error_reporting.rs  # Pulse telemetry for errors
│   ├── observability.rs    # Trace event publishing
│   ├── openapi.json        # OpenAPI 3.0 spec (progenitor codegen)
│   └── e2e_tests.rs        # End-to-end tests
└── pay-api/src/
    ├── lib.rs              # Constants (methods, currencies, states, headers)
    ├── envelope.rs         # GatewayRequest/GatewayResponse wrappers
    └── bodies/
        ├── create_payment.rs
        ├── get_payment.rs
        ├── get_payment_status.rs
        └── confirm_payment.rs
```

### Core Types

**Configuration:**
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct SdkConfig {
    pub base_url: String,
    pub project_id: String,
    pub api_key: String,
    pub sdk_name: String,
    pub sdk_version: String,
    pub sdk_platform: String,
    pub bundle_id: String,
}
```

**Main Client:**
```rust
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct WalletConnectPay {
    client: OnceLock<Client>,
    config: SdkConfig,
    cached_options: RwLock<Vec<CachedPaymentOption>>,
    error_http_client: OnceLock<reqwest::Client>,
    initialized_event_sent: OnceLock<()>,
}
```

**Public Methods:**
- `get_payment_options(payment_link, accounts)` → `PaymentOptionsResponse`
- `get_required_payment_actions(payment_link, option_id, accounts)` → `Vec<Action>`
- `confirm_payment(payment_link, option_id, signatures, collected_data)` → `ConfirmPaymentResultResponse`

**JSON Wrapper (for FFI):**
```rust
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct WalletConnectPayJson {
    client: WalletConnectPay,
}
// Methods accept/return JSON strings for cross-language compatibility
```

### Error Types

```rust
// Main errors
PayError              // HTTP/API errors, timeouts
GetPaymentOptionsError    // Payment validation (expired, not found, compliance)
GetPaymentRequestError    // Action fetching errors
ConfirmPaymentError       // Payment confirmation validation

// JSON wrapper errors
PayJsonError          // JsonParse, JsonSerialize, PaymentOptions, etc.
```

### Key Patterns

**Lazy Initialization:** Uses `OnceLock` for HTTP clients
```rust
fn client(&self) -> &Client {
    self.client.get_or_init(|| {
        Client::new(&self.config.base_url)
    })
}
```

**Error Mapping:** HTTP status → domain errors
```rust
404 → NotFound
410 → Expired
422 → InvalidAccount
451 → ComplianceFailed
```

**Retry Logic:** Exponential backoff with jitter
```rust
const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 100;
// 100ms, 200ms, 400ms (±50%)
```

**Action Caching:** Options cached for action resolution
```rust
struct CachedPaymentOption {
    option_id: String,
    actions: Vec<types::Action>,
}
```

## UniFFI Bindings

### Feature Flags

```toml
# For Kotlin release
--features=android,pay,uniffi/cli

# For Swift release
--features=ios,pay,uniffi/cli
```

### Kotlin (Android)

Generated bindings in `yttrium/kotlin-bindings/`:
- `WalletConnectPay` class with async methods
- `WalletConnectPayJson` for JSON interface
- All request/response types as Kotlin data classes

Build: `just kotlin` or via `release-kotlin.yml` workflow

### Swift (iOS)

Generated in `platforms/swift/Sources/Yttrium/`:
- `WalletConnectPay` class (Sendable)
- `WalletConnectPayJson` wrapper
- Types as Swift structs with Equatable conformance

Build: `just swift` or via `release-swift.yml` workflow

### UniFFI Derive Pattern

```rust
// For data types (records)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct MyType { ... }

// For classes with methods (objects)
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct MyClient { ... }

#[cfg_attr(feature = "uniffi", uniffi::export)]
impl MyClient {
    pub fn new(...) -> Self { ... }

    #[cfg_attr(feature = "uniffi", uniffi::method)]
    pub async fn do_something(&self) -> Result<T, E> { ... }
}

// For errors
#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
pub enum MyError { ... }
```

## Release Workflows

| Workflow | Output | Use Case |
|----------|--------|----------|
| `release-kotlin.yml` | `kotlin-artifacts.zip` | Full Kotlin SDK (pay + erc6492) |
| `release-kotlin-wcpay.yml` | `kotlin-wcpay-artifacts.zip` | Pay-only Kotlin SDK |
| `release-swift.yml` | `libyttrium.xcframework.zip` | Full Swift SDK |

Triggered via workflow_dispatch with version input (e.g., `0.10.5`).
