.PHONY: build setup build-ios-bindings fetch-thirdparty setup-thirdparty test format clean local-infra local-infra-forked local-infra-7702

build:
	cargo build --release

setup: fetch-thirdparty setup-thirdparty build-debug-mode build-ios-bindings

build-debug-mode:
	cargo build

fetch-thirdparty:
	git submodule update --init

setup-thirdparty:
	cd crates/yttrium/src/contracts/ && yarn install --frozen-lockfile --immutable && yarn compile

build-ios-bindings:
	sh crates/ffi/build-rust-ios.sh
	open Package.swift

test:
	cargo test --workspace

format:
	cargo +nightly fmt --all
	cargo sort --workspace --grouped

lint:
	cargo +nightly fmt --all -- --check
	cargo clippy --all -- -D warnings -A clippy::derive_partial_eq_without_eq -D clippy::unwrap_used -D clippy::uninlined_format_args
	cargo sort --check --workspace --grouped
	cargo +nightly udeps --workspace

clean:
	cd crates/account/src/contracts && yarn clean && cd ../../../../
	cargo clean

local-infra:
	cd test/scripts/local_infra && sh local-infra.sh

local-infra-forked:
	cd test/scripts/forked_state && sh local-infra.sh

local-infra-7702:
	cd test/scripts/7702 && sh local-infra.sh