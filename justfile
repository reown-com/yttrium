setup:
  git submodule update --init --recursive
  make setup-thirdparty

# Note: requires running `just setup` first
# TODO replace `build` with `clippy` when clippy passes
devloop: build test fmt udeps

test:
  cargo test --features=full --lib --bins

test-pimlico-api:
  cargo test --features=test_pimlico_api --lib --bins

clippy:
  cargo clippy --workspace --features=full --all-targets -- -D warnings
  # cargo clippy --workspace --features=full --lib --bins --target wasm32-unknown-unknown --exclude=ffi -- -D warnings

fmt:
  cargo +nightly fmt --all

udeps:
  cargo +nightly udeps --workspace

# TODO remove `build` in-favor of `clippy` when clippy passes
build:
  cargo build --workspace --features=full --all-targets
  # cargo build --workspace--features=full --lib --bins --target wasm32-unknown-unknown --exclude=ffi
