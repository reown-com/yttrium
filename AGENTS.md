# AGENTS.md

Guidance for AI agents (Claude Code, Copilot, Cursor, etc.) working on this repository.

## Project Overview

**Yttrium** is a cross-platform Rust library for smart account abstraction, primarily focused on Ethereum with multi-chain support. It provides account abstraction primitives (ERC-4337, ERC-7702) compiled to native binaries for iOS/Android via UniFFI and to WASM for web applications.

**Status:** Pre-alpha, under heavy development.

**Core Standards:** ERC-4337 (Account Abstraction), ERC-7702 (Set EOA account code)

## Repository Structure

```
yttrium/
├── crates/
│   ├── yttrium/           # Core library - all business logic
│   │   └── src/
│   │       ├── account_client/    # Account abstraction client
│   │       ├── sign/              # Relay protocol (~21 sub-modules)
│   │       ├── chain_abstraction/ # Cross-chain support
│   │       ├── clear_signing/     # EIP-712, asset resolution
│   │       ├── erc4337/           # ERC-4337 implementation
│   │       ├── erc7579/           # ERC-7579 modules
│   │       ├── smart_accounts/    # Smart account types
│   │       └── pay/               # Payment API
│   ├── kotlin-ffi/        # Android/Kotlin FFI bindings
│   ├── pay-api/           # Payment API types (minimal crate)
│   ├── rust-sample-wallet/# Leptos web app sample
│   └── yttrium_dart/      # Flutter/Dart bindings
├── platforms/
│   └── swift/             # Swift package and bindings
├── .github/workflows/     # CI/CD and release workflows
├── .claude/skills/        # AI agent skills
│   └── rust-quality/      # Rust coding standards skill
└── justfile               # Development commands
```

## Key Commands

```bash
# Development workflow
just devloop              # Regular dev loop: lint, test, format
just ci                   # Full CI checks (includes platform builds)
just check                # Quick checks: setup, lint, test
just lint                 # cargo +nightly fmt --all + clippy

# Testing
just test                 # Run all tests with --features=full
just test-sign            # sign_client tests with backtrace
just test-pimlico-api     # Tests requiring PIMLICO_API_KEY
just test-blockchain-api  # Tests requiring REOWN_PROJECT_ID
just test-pay-e2e         # Pay module e2e tests

# Single test
cargo test --features=full --lib --bins <test_name>

# Platform builds
just swift                # Build XCFramework and Swift bindings
just kotlin               # Build Android Kotlin bindings (NDK 26+)
just wallet               # Run Leptos sample wallet in dev mode

# Infrastructure
just infra                # Local anvil + mock alto bundler
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Core Rust Library                        │
│                   (crates/yttrium/)                         │
├─────────────────────────────────────────────────────────────┤
│  Account Abstraction    │  Cross-Chain        │  Signing    │
│  ─────────────────────  │  ───────────────    │  ────────   │
│  • account_client       │  • chain_abstraction│  • sign     │
│  • erc4337              │  • evm_signing      │  • clear_   │
│  • erc7579              │                     │    signing  │
│  • smart_accounts       │  Multi-Chain:       │             │
│  • entry_point          │  • eip155 (default) │             │
│  • eip7702              │  • solana, sui      │             │
│                         │  • stacks, ton      │             │
└─────────────────────────┴─────────────────────┴─────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│  Native/UniFFI │    │     WASM      │    │    Dart FFI   │
│  (iOS/Android) │    │  (Web/TS/JS)  │    │   (Flutter)   │
└───────────────┘    └───────────────┘    └───────────────┘
```

### Feature Flags

Features are additive and control compilation:

```toml
# Platform targets
ios, android, wasm, native, uniffi

# Blockchain namespaces
eip155 (default), solana, sui, stacks, ton

# Client features
account_client, chain_abstraction_client, erc6492_client,
transaction_sponsorship_client, sign_client, pay

# Aggregates
full = ["all_platforms", "all_clients", "all_namespaces"]
```

**Default features:** `eip155`, `erc6492_client`, `chain_abstraction_client`

## Development Notes

### Coding Standards

Use the `.claude/skills/rust-quality/` skill for detailed Rust patterns. Key points:

- **Linting:** `just lint` runs `cargo +nightly fmt --all` + clippy with `-D warnings`
- **Line width:** 80 characters max
- **Imports:** Block style, grouped by crate/external/std
- **Errors:** Use `thiserror` with `#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]`
- **FFI types:** Add `#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]`
- **Tests:** Inline `#[cfg(test)] mod tests` blocks
- **Results:** Use `eyre::Result<T>` for error propagation, avoid `unwrap()`
- **Conciseness:** Avoid extra variable bindings, minimize comments

### Environment Variables

```bash
# Pimlico tests
PIMLICO_API_KEY, PIMLICO_BUNDLER_URL, PIMLICO_RPC_URL

# Blockchain API tests
REOWN_PROJECT_ID

# Tests needing funds
FAUCET_MNEMONIC

# Android builds
ANDROID_NDK_HOME
```

### Playwright Tests

Avoid `waitForTimeout()` unless absolutely necessary.

## Versioning & Publishing

### Version Management

- Workspace version defined in root `Cargo.toml`: `version = "0.1.0"`
- Release versions are specified manually via GitHub Actions workflow dispatch
- Git tags created per release (e.g., `0.10.4`, `kotlin-wcpay-v0.1.0`)

### Release Workflows

All releases are triggered via **workflow_dispatch** with a version input:

| Workflow | Output | Features | Tag Format |
|----------|--------|----------|------------|
| `release-kotlin.yml` | `kotlin-artifacts.zip` | android, erc6492_client, pay | `v{VERSION}` |
| `release-kotlin-wcpay.yml` | `kotlin-wcpay-artifacts.zip` | android, pay | `kotlin-wcpay-v{VERSION}` |
| `release-kotlin-utils.yml` | `kotlin-utils-artifacts.zip` | android, chain_abstraction, multi-chain | `kotlin-utils-v{VERSION}` |
| `release-kotlin-all.yml` | All three above | Combined | Multiple tags |
| `release-swift.yml` | `libyttrium.xcframework.zip` | iOS, uniffi | `{VERSION}` |
| `release-swift-utils.yml` | Utils XCFramework | iOS, chain_abstraction | `swift-utils-v{VERSION}` |
| `release-dart.yml` | Dart package | dart | Per workflow |

### Publishing Targets

- **Android/Kotlin:** JitPack (via GitHub releases)
- **iOS/Swift:** Swift Package Manager + CocoaPods (via GitHub releases)
- **Dart/Flutter:** pub.dev (via release workflow)
- **WASM/npm:** Not yet automated

### Release Process

1. Ensure `main` branch is stable and CI passes
2. Go to GitHub Actions → Select release workflow
3. Click "Run workflow" → Enter version (e.g., `0.10.5`)
4. Workflow builds artifacts, creates GitHub release, uploads assets
5. For Swift: PR is created to update `Package.swift` and `YttriumWrapper.podspec`

## AI Agent Skills

Located in `.claude/skills/`:

- **rust-quality/** - Rust coding standards following yttrium patterns
  - Block imports, thiserror errors, uniffi/wasm derives
  - Newtype patterns, async patterns, testing conventions
