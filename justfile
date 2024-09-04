setup:
  git submodule update --init --recursive
  make setup-thirdparty

devloop: setup clippy test fmt udeps

test:
  cargo test --all-features --lib --bins

clippy:
  cargo clippy --workspace --all-features --all-targets -- -D warnings
  cargo clippy --workspace --all-features --all-targets --target wasm32-unknown-unknown --workspace --exclude=ffi -- -D warnings

fmt:
  cargo +nightly fmt --all

udeps:
  cargo +nightly udeps --workspace

# TODO remove in-favor of just using clippy
build:
  cargo build
  cargo build --target wasm32-unknown-unknown --workspace --exclude=ffi
