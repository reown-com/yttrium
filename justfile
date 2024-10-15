clean:
  cargo clean
  rm -rf crates/yttrium/.foundry
  rm -rf .build
  git submodule deinit --all

setup:
  git submodule update --init --recursive

devloop: setup clippy test fmt udeps

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
  # cargo +nightly udeps --workspace
