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
check: setup lint test

# Run this regularly locally, requires some special env vars
devloop: check env-tests _pass

# Devloop, but also runs all checks that CI does
devloop-ci: check env-tests _ci _pass

# devloop-ci, but also runs costly tests
devloop-ci-costly: check env-tests _ci costly-tests _pass
full: devloop-ci-costly

_ci: udeps swift kotlin

# Run all checks that CI does; helpful to autofix and help debug most CI errors
ci: check _ci _pass

setup:
  git submodule update --init --recursive

test:
  cargo test --features=full --lib --bins

# Runs tests that require environment variables to be set
env-tests:
  if [ ! -z "${PIMLICO_API_KEY}" ] && [ ! -z "${PIMLICO_RPC_URL}" ] && [ ! -z "${PIMLICO_BUNDLER_URL}" ]; then just test-pimlico-api; fi

# Runs tests that require some minor cost e.g. mainnet gas or tokens
costly-tests:
  if [ ! -z "${REOWN_PROJECT_ID}" ]; then just test-blockchain-api; fi

test-pimlico-api:
  cargo test --features=test_pimlico_api --lib --bins pimlico

test-blockchain-api:
  RUST_BACKTRACE=1 RUST_LOG=yttrium=trace cargo test --features=test_blockchain_api --lib --bins chain_abstraction::tests
  RUST_BACKTRACE=1 RUST_LOG=yttrium=trace cargo test --features=test_blockchain_api,solana --lib --bins chain_abstraction::solana::tests
test-blockchain-api-debug:
  RUST_BACKTRACE=1 RUST_LOG=yttrium=trace cargo test -p yttrium --features=test_blockchain_api chain_abstraction::tests::happy_path_execute_method -- --nocapture
test-blockchain-api-debug-solana:
  RUST_BACKTRACE=1 RUST_LOG=yttrium=trace cargo test -p yttrium --features=test_blockchain_api,solana chain_abstraction::solana::tests::solana_happy_path -- --nocapture
test-blockchain-api-debug-uselifi:
  RUST_BACKTRACE=1 RUST_LOG=yttrium=trace cargo test -p yttrium --features=test_blockchain_api chain_abstraction::tests::happy_path_lifi -- --nocapture

canary:
  RUST_BACKTRACE=1 RUST_LOG=yttrium=trace cargo test -p yttrium --features=test_blockchain_api chain_abstraction::tests::happy_path_execute_method -- --nocapture
  RUST_BACKTRACE=1 RUST_LOG=yttrium=trace cargo test -p yttrium --features=test_blockchain_api,solana chain_abstraction::solana::tests::solana_happy_path -- --nocapture

lint: fmt clippy

clippy:
  cargo clippy --workspace --all-features --all-targets -- -D warnings
  cargo clippy -p yttrium --lib --target wasm32-unknown-unknown --features=wasm -- -D warnings

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
  # cargo ndk -t armeabi-v7a -t arm64-v8a build -p kotlin-ffi --profile=uniffi-release --features=uniffi/cli
  ./build-kotlin.sh
