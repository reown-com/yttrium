set dotenv-load

clean:
  cargo clean
  rm -rf crates/yttrium/.foundry
  rm -rf .build
  git submodule deinit --all

_pass: 
  @echo ""
  @echo ""
  @echo "PASS"

# Quick config-free checks/tests
check: setup lint test _pass

# Run this regularly locally, requires some special env vars
devloop: check env-tests _pass

# Devloop, but also runs what CI does
devloop-full: check env-tests ci-extended _pass

ci-extended: udeps swift kotlin

# Run all checks that CI does
ci: check ci-extended _pass

setup:
  git submodule update --init --recursive

test:
  cargo test --features=full --lib --bins

# Runs tests that require environment variables to be set
env-tests:
  if [ ! -z "${REOWN_PROJECT_ID}" ]; then just test-blockchain-api; fi
  if [ ! -z "${PIMLICO_API_KEY}" ] && [ ! -z "${PIMLICO_RPC_URL}" ] && [ ! -z "${PIMLICO_BUNDLER_URL}" ]; then just test-pimlico-api; fi

test-pimlico-api:
  cargo test --features=test_pimlico_api --lib --bins pimlico

test-blockchain-api:
  RUST_BACKTRACE=1 cargo test --features=test_blockchain_api --lib --bins chain_abstraction::tests
test-blockchain-api-debug:
  RUST_BACKTRACE=1 cargo test --features=test_blockchain_api --lib --bins chain_abstraction::tests -- --nocapture

lint: fmt clippy

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
  make build-xcframework
  make CONFIG=debug build-swift-apple-platforms

kotlin:
  # TODO
