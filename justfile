clean:
  cargo clean
  rm -rf crates/yttrium/.foundry
  rm -rf .build
  git submodule deinit --all

setup:
  git submodule update --init --recursive

devloop: setup clippy fmt test udeps
  @echo ""
  @echo ""
  @echo "PASS"

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

infra:
  make local-infra-forked

swift:
  make build-ios-bindings
  make CONFIG=debug build-swift-apple-platforms
