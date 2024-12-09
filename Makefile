CONFIG = debug
PLATFORM_IOS = iOS Simulator,id=$(call udid_for,iOS 17.5,iPhone \d\+ Pro [^M])

setup: build-debug-mode build-ios-bindings build-swift-apple-platforms

build-swift-apple-platforms:
	export USE_LOCAL_RUST_XCFRAMEWORK=1; \
	for platform in "iOS"; do \
		xcodebuild \
			-skipMacroValidation \
			-configuration $(CONFIG) \
			-workspace .github/package.xcworkspace \
			-scheme yttrium \
			-destination generic/platform="$$platform" || exit 1; \
	done;

test-swift-apple-platforms:
	for platform in "$(PLATFORM_IOS)" ; do \
		xcodebuild test \
			-skipMacroValidation \
			-configuration $(CONFIG) \
			-workspace .github/package.xcworkspace \
			-scheme yttrium \
			-destination platform="$$platform" || exit 1; \
	done;

build-xcframework:
	sh scripts/build-xcframework.sh

set-up-local-swift-package:
	sh scripts/set-up-local-package.sh

clean:
	cd crates/account/src/contracts && yarn clean && cd ../../../../
	cargo clean

local-infra:
	cd test/scripts/local_infra && sh local-infra.sh

local-infra-forked:
	cd test/scripts/forked_state && sh local-infra.sh

local-infra-7702:
	cd test/scripts/7702 && sh local-infra.sh


.PHONY: compute-rust-checksum
compute-rust-checksum:
	swift package compute-checksum Output/RustXcframework.xcframework.zip > rust_checksum.txt
	echo "Rust XCFramework checksum: $$(cat rust_checksum.txt)"

.PHONY: generate-package-swift
generate-package-swift:
	chmod +x scripts/generate-package-swift.sh
	./scripts/generate-package-swift.sh

.PHONY: build setup build-ios-bindings build-swift-apple-platforms test-swift-apple-platforms fetch-thirdparty setup-thirdparty test format clean local-infra local-infra-forked local-infra-7702

define udid_for
$(shell xcrun simctl list devices available '$(1)' | grep '$(2)' | sort -r | head -1 | awk -F '[()]' '{ print $$(NF-3) }')
endef
