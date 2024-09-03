setup:
  git submodule update --init --recursive
  make setup-thirdparty

devloop: setup clippy test fmt udeps

test:
  cargo test --all-features --lib --bins

clippy:
  cargo clippy --workspace --all-features --all-targets -- -D warnings

fmt:
  cargo +nightly fmt --all

udeps:
  cargo +nightly udeps --workspace
