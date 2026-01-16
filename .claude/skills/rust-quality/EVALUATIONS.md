# Rust Quality Skill Evaluations

Test prompts to verify skill activation and behavior.

## Activation Tests (should trigger)

### Test 1: New error type
**Prompt:** "Add an error type for wallet connection failures with variants for timeout, invalid address, and network issues"

**Expected behavior:**
- Creates error enum with `#[derive(Debug, thiserror::Error)]`
- Adds `#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]`
- Uses descriptive `#[error("...")]` messages
- Follows naming convention (WalletConnectionError)

### Test 2: Cross-platform struct
**Prompt:** "Create a SessionConfig struct with endpoint URL, timeout duration, and retry count that works on iOS, Android, and WASM"

**Expected behavior:**
- Derives Serialize, Deserialize, Clone, Debug
- Adds uniffi::Record cfg_attr
- Adds tsify derives with cfg_attr for wasm
- Uses block-style imports
- Includes constructor and inline tests

### Test 3: New module with feature gate
**Prompt:** "Add a new notifications module that should only compile when the notifications feature is enabled"

**Expected behavior:**
- Creates module with `#[cfg(feature = "notifications")]` in lib.rs
- Uses proper import grouping
- Follows impl block organization (constructor, getters, async ops)
- Includes `#[cfg(test)] mod tests` block

### Test 4: Async function
**Prompt:** "Write an async function to fetch user balance from an RPC endpoint"

**Expected behavior:**
- Returns `eyre::Result<T>`
- Uses `?` for error propagation
- No `unwrap()` calls
- Follows function naming conventions

---

## Non-Activation Tests (should NOT trigger)

### Test 5: Python code
**Prompt:** "Write a Python function to parse JSON"

**Expected behavior:**
- Does NOT apply Rust patterns
- Writes idiomatic Python code
- No mention of thiserror, uniffi, or Cargo features

### Test 6: Documentation edit
**Prompt:** "Fix the typo in README.md"

**Expected behavior:**
- Simply fixes the typo
- Does NOT restructure or add Rust-specific patterns

### Test 7: JavaScript code
**Prompt:** "Add a new React component for displaying wallet balance"

**Expected behavior:**
- Writes idiomatic React/JavaScript
- No Rust patterns applied

---

## Edge Case Tests

### Test 8: Mixed codebase
**Prompt:** "This project has both Rust and TypeScript. Add a new type that needs to work in both."

**Expected behavior:**
- For Rust side: applies rust-quality patterns
- For TypeScript side: writes idiomatic TS
- Ensures type compatibility between the two
- Uses wasm-bindgen/tsify patterns for interop

### Test 9: Existing code modification
**Prompt:** "Refactor this existing function to handle errors better" (given code without proper error handling)

**Expected behavior:**
- Converts to `eyre::Result<T>` return type
- Replaces `unwrap()` with `?`
- Creates error enum if appropriate
- Maintains existing functionality

### Test 10: Performance-critical code
**Prompt:** "Optimize this hot path for minimal allocations"

**Expected behavior:**
- Still follows code style (imports, formatting)
- May relax some patterns (e.g., skip unnecessary clones)
- Documents performance trade-offs if deviating from standard patterns
- Uses `#[inline]` where appropriate

### Test 11: No uniffi needed
**Prompt:** "Add an internal utility function that's only used within the crate"

**Expected behavior:**
- Does NOT add uniffi derives (internal only)
- Does NOT add wasm derives (internal only)
- Keeps impl simpler without FFI concerns
- Still follows error handling and import patterns

---

## Validation Checklist Tests

### Test 12: Full module creation
**Prompt:** "Create a new caching module with a thread-safe provider pool"

**Validation:**
- [ ] Block-style imports used
- [ ] Error type with thiserror (if errors needed)
- [ ] Proper cfg_attr for uniffi/wasm (if exposed)
- [ ] Newtype with full derives (if custom types)
- [ ] Inline tests present
- [ ] No clippy warnings when running `cargo clippy`
- [ ] Lines under 80 characters
- [ ] Uses Arc<RwLock<_>> for thread-safe state
