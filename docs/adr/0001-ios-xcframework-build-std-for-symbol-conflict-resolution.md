# ADR-0001: Use -Z build-std for iOS XCFramework Builds to Eliminate Rust Symbol Conflicts

## Status

Accepted

## Date

2026-01-15

## Context

Yttrium is a cross-platform Rust library that compiles to iOS XCFrameworks via UniFFI. When iOS applications include multiple Rust libraries (e.g., Yttrium alongside another Rust-based SDK), linker errors occur due to duplicate symbols like `rust_eh_personality`.

The `rust_eh_personality` symbol is part of Rust's panic handling/unwinding mechanism. When the standard library is compiled with the default panic strategy, these symbols are included in every Rust static library. Multiple Rust libraries in the same iOS project result in duplicate symbol errors at link time.

This is a known issue in the Rust ecosystem when multiple Rust crates are linked into a single binary on Apple platforms. The standard mitigation approaches (like using `panic = "abort"` in Cargo profiles) do not completely eliminate these symbols because the pre-compiled standard library still contains them.

## Decision Drivers

- iOS applications using multiple Rust libraries cannot link successfully due to duplicate symbols
- The solution must work with the existing UniFFI-based FFI generation pipeline
- Kotlin/Android builds should not be affected by this change
- The solution should completely eliminate the problematic symbols, not just reduce them
- Build times and complexity should remain manageable

## Considered Options

### Option 1: Use `panic = "abort"` in Cargo Profile Only

Set `panic = "abort"` in the release profile without rebuilding the standard library.

**Pros:**
- Simple configuration change
- Works with stable Rust toolchain
- No additional toolchain requirements

**Cons:**
- Does not eliminate `rust_eh_personality` from the pre-compiled standard library
- Duplicate symbol conflicts persist with other Rust libraries
- Only reduces user code's panic handling, not stdlib

### Option 2: Use `-Z build-std` with Nightly Rust

Rebuild the Rust standard library from source with `-Z build-std=std,panic_abort` and use `panic_immediate_abort` feature to completely eliminate panic handling code.

**Pros:**
- Completely eliminates `rust_eh_personality` and related symbols
- Solves the duplicate symbol problem definitively
- Standard library is built with consistent panic strategy

**Cons:**
- Requires nightly Rust toolchain
- Requires `rust-src` component
- Longer build times (stdlib rebuilt from source)
- Uses unstable compiler flags

### Option 3: Link Rust Libraries Dynamically

Use dynamic libraries instead of static libraries to avoid symbol conflicts.

**Pros:**
- Avoids static linking duplicate symbol issues
- No special compiler flags needed

**Cons:**
- Significantly complicates iOS app distribution
- Dynamic frameworks have runtime overhead
- Not well-supported for Rust on iOS
- Would require major changes to the FFI architecture

## Decision

We decided on **Option 2: Use `-Z build-std` with Nightly Rust** because it is the only option that completely eliminates the problematic symbols while maintaining the existing static library approach.

The implementation includes:

1. **New Cargo profile** (`xcframework-release`) with `panic = "abort"`, LTO, and stripping enabled
2. **Build script changes** to use `cargo +nightly build` with:
   - `-Z build-std=std,panic_abort` to rebuild stdlib without unwinding
   - `-Z unstable-options` for additional nightly features
   - `-Zunstable-options -Cpanic=immediate-abort` in RUSTFLAGS for complete elimination
3. **CI workflow updates** to install nightly toolchain with `rust-src` component and iOS targets

The Kotlin/Android builds continue to use the stable toolchain and are not affected.

## Consequences

### Positive

- iOS applications can now include Yttrium alongside other Rust libraries without linker errors
- Symbol table is significantly smaller due to elimination of panic handling code
- Existing `uniffi-release-swift` profile preserved as fallback if needed
- Clear separation of concerns: iOS uses nightly with build-std, Android uses stable

### Negative

- Build environment requires nightly Rust toolchain for iOS builds
- Longer build times due to standard library recompilation
- Dependency on unstable Rust compiler features (`-Z build-std`)
- If Rust stabilizes a different approach, migration may be needed

### Neutral

- Developers building locally for iOS need nightly toolchain with rust-src
- CI workflows now explicitly manage toolchain installation

## Validation Rules

```yaml
rules:
  - id: "0001-ios-xcframework-nightly"
    description: "iOS XCFramework build scripts must use nightly toolchain with build-std"
    pattern: "cargo\\s+build.*--profile.*xcframework"
    action: "warn"
    applies_to: ["scripts/build-xcframework.sh", "scripts/build-utils-xcframework.sh"]
    note: "If this pattern matches without '+nightly' and '-Z build-std', the build may produce symbol conflicts"

  - id: "0001-xcframework-profile-panic"
    description: "xcframework-release profile must use panic=abort"
    pattern: "\\[profile\\.xcframework-release\\]"
    action: "warn"
    applies_to: ["Cargo.toml"]
    note: "Verify this profile includes panic = 'abort' for symbol elimination"
```

## Implementation Notes

### Building Locally

```bash
# Ensure nightly toolchain with rust-src is installed
rustup toolchain install nightly --component rust-src
rustup target add aarch64-apple-ios --toolchain nightly
rustup target add x86_64-apple-ios --toolchain nightly
rustup target add aarch64-apple-ios-sim --toolchain nightly

# Build XCFramework
make build-xcframework
```

### Verifying Symbol Elimination

```bash
# Check that rust_eh_personality is not present
nm -g target/ios/libyttrium.xcframework/ios-arm64/libyttrium.a | grep rust_eh_personality
# Should return empty (no matches)
```

### Key Files Changed

- `Cargo.toml`: New `xcframework-release` profile
- `scripts/build-xcframework.sh`: Nightly + build-std flags
- `scripts/build-utils-xcframework.sh`: Same build configuration
- `.github/workflows/release-swift.yml`: Nightly toolchain installation
- `.github/workflows/release-swift-utils.yml`: Same CI changes

## Related

- References: [Notion Technical Doc](https://www.notion.so/walletconnect/Yttrium-Rust-Symbol-Conflicts-Technical-Analysis-and-Resolution-2e93a661771e803aa849e1efb98e56c5)
- References: [Rust build-std documentation](https://doc.rust-lang.org/cargo/reference/unstable.html#build-std)
- References: [PR #331](https://github.com/reown-com/yttrium/pull/331)
