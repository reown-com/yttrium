# Clear Signing Library — Current State (Dec 2025)

## TL;DR

- Rust core + FFI already format Ledger-style intents for ERC-20 approvals, WETH, Aave v2/v3, 1inch routers, and Permit2 typed data (`crates/yttrium/src/clear_signing/mod.rs`, `engine.rs`).
- All descriptors/ABIs/tokens/address labels live under `crates/yttrium/src/clear_signing/assets/*`; resolver + token registry never fetch from the network.
- Interpolated intents are rendered end-to-end (unlike Ledger today) and are exercised by the calldata + typed-data tests in `crates/yttrium/tests/clear_signing.rs`.
- Gaps: limited descriptor/token coverage, no Safe recursion, unsigned registries, no EIP-5792, and registries mirror only our local fork.

---

## State of the Code

- **Resolver + Engine**: `format`/`format_with_value` drive the engine, which decodes ABI inputs, applies descriptor-defined formats (including `interpolatedIntent` templates), and falls back to raw previews/warnings when selectors are unknown.
- **Registries**: `assets/index.json` and `index_eip712.json` map CAIP-10 keys to Ledger-derived descriptors (Aave, 1inch, ERC-20, WETH9, Stakeweight, Permit2). `tokens-min.json` only covers ETH/USDC/USDT/WETH on four chains. `address_book.json` seeds a handful of spender labels; 
- **Bindings**: `uniffi` exports `clear_signing_format`, `_with_value`, and `_typed`; Swift already consumes them (`platforms/swift/.../yttrium.swift`), and Kotlin/JS can do the same once their wrappers flip on the feature.
- **Tests**: `cargo test -p yttrium clear_signing` runs the approval/swap/Aave/Permit2 

---

## What’s Missing 

1. **Descriptor coverage** — only ~30 CAIP-10 keys exist in `index.json`, all for 1inch, Aave, ERC-20, and our WalletConnect demo. We still lack descriptors for Safe, Uniswap Universal Router, Lido, Pendle, EigenLayer, bridges, staking, NFT marketplaces, etc.
2. **Nested / interpolated intents** — we render interpolated strings today, but Ledger’s upstream registry does not yet publish `interpolatedIntent` fields. We only maintain them in our forked descriptors.
3. **Token registry** — `tokens-min.json` only has ETH/USDC/USDT/WETH on four chains. Any other CAIP-19 lookup errors out. 
4. **Address labels** — the shared `address_book.json` only holds three spenders today. Every unknown address shows as hex unless the descriptor explicitly labels it.
5. **Typed-data coverage** — only Permit2 + 1inch limit orders. 
6. **Safe / Recursive calldata** — the resolver decodes exactly one call; there is no recursion or typed detection for `Safe.execTransaction` payloads yet, so nested calls just appear as raw bytes.
7. **Descriptor provenance** — descriptors and tickers are unsigned and bundled in the binary. There is no signature verification, hash pinning, or remote fetch.
8. **Tooling** — no generator to pull upstream Ledger descriptors, dedupe proxies, or run schema validation. Updates require manual editing of JSON under `assets/`.

---

## Using the Library Today

```rust
use yttrium::clear_signing::format_with_value;

let calldata = hex::decode("0x095ea7b3...")?;
let preview = format_with_value(
    1,
    "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT
    None,
    &calldata,
)?;

assert_eq!(preview.intent, "Approve USDT spending");
println!("{:?}", preview.items);
```

```rust
use yttrium::clear_signing::{format_typed_data, TypedData};

let typed: TypedData = serde_json::from_str(&permit2_json)?;
let preview = format_typed_data(&typed)?;
assert_eq!(preview.intent, "Authorize spending of token");
```

```swift
let preview = try clearSigningFormatWithValue(
    chainId: 1,
    to: "0xdAC17F958D2ee523a2206206994597C13D831ec7",
    valueHex: nil,
    calldataHex: "0x095ea7b3000000000000..."
)
print(preview.intent)
```

---

## Next Steps (Recommended Before Customer Handoff)

1. **Generate full descriptor + token bundles** from Ledger’s registry, add missing deployments, and document the sync process.
2. **Expose SDK hooks for wallet-managed metadata** so integrators can feed their own address labels
