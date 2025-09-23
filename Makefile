CONFIG = debug
PLATFORM_IOS = iOS Simulator,id=$(call udid_for,iOS 17.5,iPhone \d\+ Pro [^M])

build:
	# GitHub CodeQL is automatically calling `make build` on PRs
	# This Makefile is deprecated, but we need to keep this make target for CodeQL to work
	cargo build

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

build-utils-xcframework:
	sh scripts/build-utils-xcframework.sh

set-up-local-swift-package:
	sh scripts/set-up-local-swift-package.sh

clean:
	cd crates/account/src/contracts && yarn clean && cd ../../../../
	cargo clean

local-infra:
	cd test/scripts/local_infra && sh local-infra.sh

local-infra-forked:
	cd test/scripts/forked_state && sh local-infra.sh

local-infra-7702:
	cd test/scripts/7702 && sh local-infra.sh

.PHONY: generate-package-swift-utils
generate-package-swift-utils:
	chmod +x scripts/generate-package-swift-utils.sh
	./scripts/generate-package-swift-utils.sh

.PHONY: update-package-swift-core
update-package-swift-core:
	chmod +x scripts/update-package-swift-core.sh
	@echo "Usage: make update-package-swift-core VERSION=0.9.46 CHECKSUM=abc123..."
	@if [ -z "$(VERSION)" ] || [ -z "$(CHECKSUM)" ]; then \
		echo "Error: VERSION and CHECKSUM are required"; \
		echo "Example: make update-package-swift-core VERSION=0.9.46 CHECKSUM=abc123..."; \
		exit 1; \
	fi
	./scripts/update-package-swift-core.sh $(VERSION) $(CHECKSUM)

.PHONY: update-package-swift-utils
update-package-swift-utils:
	chmod +x scripts/update-package-swift-utils.sh
	@echo "Usage: make update-package-swift-utils VERSION=0.0.2 CHECKSUM=xyz789..."
	@if [ -z "$(VERSION)" ] || [ -z "$(CHECKSUM)" ]; then \
		echo "Error: VERSION and CHECKSUM are required"; \
		echo "Example: make update-package-swift-utils VERSION=0.0.2 CHECKSUM=xyz789..."; \
		exit 1; \
	fi
	./scripts/update-package-swift-utils.sh $(VERSION) $(CHECKSUM)

.PHONY: build build-ios-bindings build-swift-apple-platforms test-swift-apple-platforms fetch-thirdparty setup-thirdparty test format clean local-infra local-infra-forked local-infra-7702

define udid_for
$(shell xcrun simctl list devices available '$(1)' | grep '$(2)' | sort -r | head -1 | awk -F '[()]' '{ print $$(NF-3) }')
endef
