# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Primary development commands
just devloop         # Regular development loop: lint, test, format fixes
just ci              # Full CI checks (includes platform builds)
just check           # Quick checks: setup, lint, test
just lint            # Run cargo fmt + clippy

# Testing
just test            # Run all tests with full features
just test-sign       # Run sign_client tests with backtrace
just test-pimlico-api    # Tests requiring PIMLICO_API_KEY env vars
just test-blockchain-api # Tests requiring REOWN_PROJECT_ID

# Platform builds
just swift           # Build XCFramework and Swift bindings
just kotlin          # Build Android Kotlin bindings (requires NDK 26+)
just wallet          # Run Leptos sample wallet in dev mode

# Run a single test
cargo test --features=full --lib --bins test_name

# Local infrastructure (required by some tests)
just infra           # Runs local anvil + mock alto bundler
```

## Architecture

Yttrium is a cross-platform Rust library for smart account abstraction, primarily focused on Ethereum with multi-chain support.

```
Core Rust Library (crates/yttrium/)
├── Compiled to Native → Consumed by Swift/Kotlin via UniFFI
├── Compiled to WASM   → Consumed by TypeScript/JavaScript
└── Sample implementations (Leptos web wallet, Flutter)

crates/
├── yttrium/           # Core library with all business logic
├── kotlin-ffi/        # Android/Kotlin FFI bindings
├── pay-api/           # Payment API types & interfaces
├── rust-sample-wallet/# Leptos web app sample
└── yttrium_dart/      # Flutter/Dart bindings
```

### Core Modules (in `crates/yttrium/src/`)

**Account Abstraction**: `account_client`, `erc4337`, `erc7579`, `erc6492_client`, `eip7702`, `smart_accounts`, `entry_point`

**Cross-Chain**: `chain_abstraction` (blockchain API integration), `evm_signing`

**Signing**: `sign` (relay protocol with WebSocket, ~21 sub-modules), `clear_signing` (EIP-712, asset resolution)

**Multi-Chain Support**: Ethereum (default), Solana, SUI, Stacks, TON

## Feature Flags

Features are additive and control what gets compiled:

```toml
# Platform targets
ios, android, wasm, native, uniffi

# Blockchain namespaces
eip155 (default), solana, sui, stacks, ton

# Client features
account_client, chain_abstraction_client, erc6492_client,
transaction_sponsorship_client, sign_client, pay

# Aggregate features
full = ["all_platforms", "all_clients", "all_namespaces"]
```

Default: `eip155`, `erc6492_client`, `chain_abstraction_client`

## Coding Standards

- Use `just lint` for linting (runs `cargo +nightly fmt --all` + clippy)
- Avoid creating functions unless they reduce significant code duplication
- Avoid extra variable bindings
- Keep code and tests concise - minimize comments unless important for readability
- For Playwright tests: avoid `waitForTimeout()` unless absolutely necessary

## Environment Variables

Required for specific test suites:
```
PIMLICO_API_KEY, PIMLICO_BUNDLER_URL, PIMLICO_RPC_URL  # Pimlico tests
FAUCET_MNEMONIC                                         # Tests needing funds
REOWN_PROJECT_ID                                        # Blockchain API tests
ANDROID_NDK_HOME                                        # Android builds
```

## Project Status

Pre-alpha, under heavy development. Core standards: ERC-4337, ERC-7702.
