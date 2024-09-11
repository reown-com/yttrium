CONFIG = debug
PLATFORM_IOS = iOS Simulator,id=$(call udid_for,iOS 17.5,iPhone \d\+ Pro [^M])

build:
	cargo build --release

setup: fetch-thirdparty setup-thirdparty build-debug-mode build-ios-bindings build-swift-apple-platforms

build-swift-apple-platforms:
	for platform in "iOS"; do \
		xcodebuild \
			-configuration $(CONFIG) \
			-workspace .github/package.xcworkspace \
			-scheme yttrium \
			-destination generic/platform="$$platform" || exit 1; \
	done;

test-swift-apple-platforms:
	for platform in "$(PLATFORM_IOS)" ; do \
		xcodebuild test \
			-configuration $(CONFIG) \
			-workspace .github/package.xcworkspace \
			-scheme yttrium \
			-destination platform="$$platform" || exit 1; \
	done;

build-debug-mode:
	cargo build

fetch-thirdparty:
	git submodule update --init

setup-thirdparty:
	cd crates/yttrium/src/contracts/ && yarn install --frozen-lockfile --immutable && yarn compile
	cd crates/yttrium/safe-smart-account/ && npm install
	cd crates/yttrium/safe-modules/ && pnpm install

build-ios-bindings:
	sh crates/ffi/build-rust-ios.sh

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

.PHONY: build setup build-ios-bindings build-swift-apple-platforms test-swift-apple-platforms fetch-thirdparty setup-thirdparty test format clean local-infra local-infra-forked local-infra-7702

define udid_for
$(shell xcrun simctl list devices available '$(1)' | grep '$(2)' | sort -r | head -1 | awk -F '[()]' '{ print $$(NF-3) }')
endef
