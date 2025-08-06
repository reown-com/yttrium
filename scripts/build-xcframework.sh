#!/usr/bin/env zsh

set -e
set -u

PACKAGE_NAME="yttrium"
FEATURES="ios,erc6492_client"
fat_simulator_lib_dir="target/ios-simulator-fat/uniffi-release-swift"
swift_package_dir="platforms/swift/Sources/Yttrium"

build_rust_libraries() {
  #### Building for aarch64-apple-ios (Physical Devices) ####
  echo "Building for aarch64-apple-ios..."

  # Set environment variables
  export CC_aarch64_apple_ios="$(xcrun --sdk iphoneos --find clang)"
  export AR_aarch64_apple_ios="$(xcrun --sdk iphoneos --find ar)"
  export CARGO_TARGET_AARCH64_APPLE_IOS_LINKER="$CC_aarch64_apple_ios"
  export RUSTFLAGS="-C linker=$CC_aarch64_apple_ios -C link-arg=-miphoneos-version-min=13.0"

  # Build
  cargo build \
    --lib --profile=uniffi-release-swift \
    --no-default-features \
    --features=$FEATURES \
    --target aarch64-apple-ios \
    -p yttrium

  # Unset environment variables
  unset CC_aarch64_apple_ios
  unset AR_aarch64_apple_ios
  unset CARGO_TARGET_AARCH64_APPLE_IOS_LINKER
  unset RUSTFLAGS

  #### Building for x86_64-apple-ios (Simulator on Intel Macs) ####
  echo "Building for x86_64-apple-ios..."

  # Set environment variables
  export CC_x86_64_apple_ios="$(xcrun --sdk iphonesimulator --find clang)"
  export AR_x86_64_apple_ios="$(xcrun --sdk iphonesimulator --find ar)"
  export CARGO_TARGET_X86_64_APPLE_IOS_LINKER="$CC_x86_64_apple_ios"
  export RUSTFLAGS="-C linker=$CC_x86_64_apple_ios -C link-arg=-mios-simulator-version-min=13.0"

  cargo build \
    --lib --profile=uniffi-release-swift \
    --no-default-features \
    --features=$FEATURES \
    --target x86_64-apple-ios \
    -p yttrium

  # Unset environment variables
  unset CC_x86_64_apple_ios
  unset AR_x86_64_apple_ios
  unset CARGO_TARGET_X86_64_APPLE_IOS_LINKER
  unset RUSTFLAGS

  #### Building for aarch64-apple-ios-sim (Simulator on Apple Silicon Macs) ####
  echo "Building for aarch64-apple-ios-sim..."

  # Set environment variables
  export CC_aarch64_apple_ios_sim="$(xcrun --sdk iphonesimulator --find clang)"
  export AR_aarch64_apple_ios_sim="$(xcrun --sdk iphonesimulator --find ar)"
  export CARGO_TARGET_AARCH64_APPLE_IOS_SIM_LINKER="$CC_aarch64_apple_ios_sim"
  export RUSTFLAGS="-C linker=$CC_aarch64_apple_ios_sim -C link-arg=-mios-simulator-version-min=13.0"

  cargo build \
    --lib --profile=uniffi-release-swift \
    --no-default-features \
    --features=$FEATURES \
    --target aarch64-apple-ios-sim \
    -p yttrium

  # Unset environment variables
  unset CC_aarch64_apple_ios_sim
  unset AR_aarch64_apple_ios_sim
  unset CARGO_TARGET_AARCH64_APPLE_IOS_SIM_LINKER
  unset RUSTFLAGS
}

generate_ffi() {
  echo "Generating framework module mapping and FFI bindings..."
  cargo run -p yttrium --no-default-features --features=$FEATURES,uniffi/cli --bin uniffi-bindgen generate \
    --library target/aarch64-apple-ios/uniffi-release-swift/lib$PACKAGE_NAME.dylib \
    --language swift \
    --out-dir target/uniffi-xcframework-staging
}

build_xcframework() {
  echo "Generating XCFramework..."
  rm -rf target/ios
  mkdir -p target/ios

  # Create headers directory structure for device
  mkdir -p target/uniffi-xcframework-staging/device/Headers/yttriumFFI
  cp -r target/uniffi-xcframework-staging/yttriumFFI/* target/uniffi-xcframework-staging/device/Headers/yttriumFFI/

  # Create headers directory structure for simulator
  mkdir -p target/uniffi-xcframework-staging/simulator/Headers/yttriumFFI
  cp -r target/uniffi-xcframework-staging/yttriumFFI/* target/uniffi-xcframework-staging/simulator/Headers/yttriumFFI/

  xcodebuild -create-xcframework \
      -library "target/aarch64-apple-ios/uniffi-release-swift/lib$1.a" -headers target/uniffi-xcframework-staging/device/Headers \
      -library "$fat_simulator_lib_dir/lib$1.a" -headers target/uniffi-xcframework-staging/simulator/Headers \
      -output "target/ios/lib$1.xcframework"
}

create_fat_simulator_lib() {
  echo "Creating a fat library for x86_64 and aarch64 simulators..."
  mkdir -p "$fat_simulator_lib_dir"
  lipo -create \
      "target/x86_64-apple-ios/uniffi-release-swift/lib$1.a" \
      "target/aarch64-apple-ios-sim/uniffi-release-swift/lib$1.a" \
      -output "$fat_simulator_lib_dir/lib$1.a"
}

# Add the necessary Rust targets
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios
rustup target add aarch64-apple-ios-sim

# Execute the build steps
build_rust_libraries
generate_ffi $PACKAGE_NAME
create_fat_simulator_lib $PACKAGE_NAME
build_xcframework $PACKAGE_NAME
