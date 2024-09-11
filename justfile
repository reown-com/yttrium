setup:
  git submodule update --init --recursive
  make setup-thirdparty

# Note: requires running `just setup` first
# TODO replace `build` with `clippy` when clippy passes
devloop: build test fmt udeps

test:
  cargo test --all-features --lib --bins

clippy:
  cargo clippy --workspace --all-features --all-targets -- -D warnings
  # cargo clippy --workspace --all-features --lib --bins --target wasm32-unknown-unknown --exclude=ffi -- -D warnings

fmt:
  cargo +nightly fmt --all

udeps:
  cargo +nightly udeps --workspace

# TODO remove `build` in-favor of `clippy` when clippy passes
build:
  cargo build --workspace --all-features --all-targets
  # cargo build --workspace --all-features --lib --bins --target wasm32-unknown-unknown --exclude=ffi
