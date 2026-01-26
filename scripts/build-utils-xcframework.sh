#!/usr/bin/env zsh

set -e
set -u

PACKAGE_NAME="yttrium"
# Use chain_abstraction_client which will include Sui code without explicit sui feature
FEATURES="ios,eip155,chain_abstraction_client,stacks,sui,ton,tron,clear_signing,evm_signing"
PROFILE="xcframework-release"
fat_simulator_lib_dir="target/ios-simulator-fat/$PROFILE-utils"
swift_package_dir="platforms/swift/Sources/YttriumUtils"
ORIG_FFI_MODULE_NAME="yttriumFFI"
UTILS_FFI_MODULE_NAME="yttriumUtilsFFI"

build_rust_libraries() {
  #### Building for aarch64-apple-ios (Physical Devices) ####
  echo "Building for aarch64-apple-ios..."

  # Set environment variables
  export CC_aarch64_apple_ios="$(xcrun --sdk iphoneos --find clang)"
  export AR_aarch64_apple_ios="$(xcrun --sdk iphoneos --find ar)"
  export CARGO_TARGET_AARCH64_APPLE_IOS_LINKER="$CC_aarch64_apple_ios"
  # Ensure C/C++ built via cc crate uses consistent min iOS version
  export IPHONEOS_DEPLOYMENT_TARGET="13.0"
  export CFLAGS_aarch64_apple_ios="-miphoneos-version-min=13.0"
  export RUSTFLAGS="-C linker=$CC_aarch64_apple_ios -C link-arg=-miphoneos-version-min=13.0"

  # Build with nightly and -Z build-std to eliminate rust_eh_personality symbols
  cargo +nightly build \
    --lib --profile=$PROFILE \
    -Z build-std=std,panic_abort \
    -Z unstable-options \
    --no-default-features \
    --features=$FEATURES \
    --target aarch64-apple-ios \
    -p yttrium

  # Unset environment variables
  unset CC_aarch64_apple_ios
  unset AR_aarch64_apple_ios
  unset CARGO_TARGET_AARCH64_APPLE_IOS_LINKER
  unset IPHONEOS_DEPLOYMENT_TARGET
  unset CFLAGS_aarch64_apple_ios
  unset RUSTFLAGS

  #### Building for x86_64-apple-ios (Simulator on Intel Macs) ####
  echo "Building for x86_64-apple-ios..."

  # Set environment variables
  export CC_x86_64_apple_ios="$(xcrun --sdk iphonesimulator --find clang)"
  export AR_x86_64_apple_ios="$(xcrun --sdk iphonesimulator --find ar)"
  export CARGO_TARGET_X86_64_APPLE_IOS_LINKER="$CC_x86_64_apple_ios"
  # Ensure C/C++ built via cc crate uses consistent min iOS Simulator version
  export IPHONEOS_DEPLOYMENT_TARGET="13.0"
  export CFLAGS_x86_64_apple_ios="-mios-simulator-version-min=13.0"
  export RUSTFLAGS="-C linker=$CC_x86_64_apple_ios -C link-arg=-mios-simulator-version-min=13.0"

  cargo +nightly build \
    --lib --profile=$PROFILE \
    -Z build-std=std,panic_abort \
    -Z unstable-options \
    --no-default-features \
    --features=$FEATURES \
    --target x86_64-apple-ios \
    -p yttrium

  # Unset environment variables
  unset CC_x86_64_apple_ios
  unset AR_x86_64_apple_ios
  unset CARGO_TARGET_X86_64_APPLE_IOS_LINKER
  unset IPHONEOS_DEPLOYMENT_TARGET
  unset CFLAGS_x86_64_apple_ios
  unset RUSTFLAGS

  #### Building for aarch64-apple-ios-sim (Simulator on Apple Silicon Macs) ####
  echo "Building for aarch64-apple-ios-sim..."

  # Set environment variables
  export CC_aarch64_apple_ios_sim="$(xcrun --sdk iphonesimulator --find clang)"
  export AR_aarch64_apple_ios_sim="$(xcrun --sdk iphonesimulator --find ar)"
  export CARGO_TARGET_AARCH64_APPLE_IOS_SIM_LINKER="$CC_aarch64_apple_ios_sim"
  # Ensure C/C++ built via cc crate uses consistent min iOS Simulator version
  export IPHONEOS_DEPLOYMENT_TARGET="13.0"
  export CFLAGS_aarch64_apple_ios_sim="-mios-simulator-version-min=13.0"
  export RUSTFLAGS="-C linker=$CC_aarch64_apple_ios_sim -C link-arg=-mios-simulator-version-min=13.0"

  cargo +nightly build \
    --lib --profile=$PROFILE \
    -Z build-std=std,panic_abort \
    -Z unstable-options \
    --no-default-features \
    --features=$FEATURES \
    --target aarch64-apple-ios-sim \
    -p yttrium

  # Unset environment variables
  unset CC_aarch64_apple_ios_sim
  unset AR_aarch64_apple_ios_sim
  unset CARGO_TARGET_AARCH64_APPLE_IOS_SIM_LINKER
  unset IPHONEOS_DEPLOYMENT_TARGET
  unset CFLAGS_aarch64_apple_ios_sim
  unset RUSTFLAGS
}

generate_ffi() {
  echo "Generating framework module mapping and FFI bindings for Utils..."
  cargo +nightly run -p yttrium --no-default-features --features=$FEATURES,uniffi/cli --bin uniffi-bindgen generate \
    --library target/aarch64-apple-ios/$PROFILE/lib$PACKAGE_NAME.dylib \
    --language swift \
    --out-dir target/uniffi-xcframework-staging-utils
}

rename_ffi_module() {
  echo "Namespacing FFI module to ${UTILS_FFI_MODULE_NAME}..."
  local staging="target/uniffi-xcframework-staging-utils"

  # Directory case
  if [ -d "$staging/$ORIG_FFI_MODULE_NAME" ]; then
    rm -rf "$staging/$UTILS_FFI_MODULE_NAME"
    mkdir -p "$staging/$UTILS_FFI_MODULE_NAME"
    cp -R "$staging/$ORIG_FFI_MODULE_NAME/." "$staging/$UTILS_FFI_MODULE_NAME/"
    # Rename header to avoid collision and update modulemap
    if [ -f "$staging/$UTILS_FFI_MODULE_NAME/yttriumFFI.h" ]; then
      mv "$staging/$UTILS_FFI_MODULE_NAME/yttriumFFI.h" "$staging/$UTILS_FFI_MODULE_NAME/${UTILS_FFI_MODULE_NAME}.h"
    fi
    # Overwrite module.modulemap to ensure correct module name and header reference
    cat > "$staging/$UTILS_FFI_MODULE_NAME/module.modulemap" <<EOF
module $UTILS_FFI_MODULE_NAME {
  header "${UTILS_FFI_MODULE_NAME}.h"
  export *
}
EOF
  else
    # Flat files case
    mkdir -p "$staging/$UTILS_FFI_MODULE_NAME"
    if [ -f "$staging/${ORIG_FFI_MODULE_NAME}.h" ]; then
      cp "$staging/${ORIG_FFI_MODULE_NAME}.h" "$staging/$UTILS_FFI_MODULE_NAME/${UTILS_FFI_MODULE_NAME}.h"
    fi
    # Create a fresh module.modulemap
    cat > "$staging/$UTILS_FFI_MODULE_NAME/module.modulemap" <<EOF
module $UTILS_FFI_MODULE_NAME {
  header "${UTILS_FFI_MODULE_NAME}.h"
  export *
}
EOF
  fi
}

build_xcframework() {
  echo "Generating YttriumUtils XCFramework..."
  rm -rf target/ios-utils
  mkdir -p target/ios-utils

  # Clean staging headers to avoid leftovers from previous runs
  rm -rf target/uniffi-xcframework-staging-utils/device target/uniffi-xcframework-staging-utils/simulator

  # Create headers directory structure for device (namespaced to avoid collisions)
  mkdir -p target/uniffi-xcframework-staging-utils/device/Headers/$UTILS_FFI_MODULE_NAME
  
  # Copy namespaced headers into namespaced folder
  if [ -d "target/uniffi-xcframework-staging-utils/$UTILS_FFI_MODULE_NAME" ]; then
    cp -R target/uniffi-xcframework-staging-utils/$UTILS_FFI_MODULE_NAME/. target/uniffi-xcframework-staging-utils/device/Headers/$UTILS_FFI_MODULE_NAME/
  else
    mkdir -p target/uniffi-xcframework-staging-utils/device/Headers/$UTILS_FFI_MODULE_NAME
    cp target/uniffi-xcframework-staging-utils/${ORIG_FFI_MODULE_NAME}.modulemap target/uniffi-xcframework-staging-utils/device/Headers/$UTILS_FFI_MODULE_NAME/module.modulemap || true
    cp target/uniffi-xcframework-staging-utils/${ORIG_FFI_MODULE_NAME}.h target/uniffi-xcframework-staging-utils/device/Headers/$UTILS_FFI_MODULE_NAME/${UTILS_FFI_MODULE_NAME}.h || true
  fi

  # Create headers directory structure for simulator (namespaced to avoid collisions)
  mkdir -p target/uniffi-xcframework-staging-utils/simulator/Headers/$UTILS_FFI_MODULE_NAME
  
  # Copy namespaced headers for simulator
  if [ -d "target/uniffi-xcframework-staging-utils/$UTILS_FFI_MODULE_NAME" ]; then
    cp -R target/uniffi-xcframework-staging-utils/$UTILS_FFI_MODULE_NAME/. target/uniffi-xcframework-staging-utils/simulator/Headers/$UTILS_FFI_MODULE_NAME/
  else
    mkdir -p target/uniffi-xcframework-staging-utils/simulator/Headers/$UTILS_FFI_MODULE_NAME
    cp target/uniffi-xcframework-staging-utils/${ORIG_FFI_MODULE_NAME}.modulemap target/uniffi-xcframework-staging-utils/simulator/Headers/$UTILS_FFI_MODULE_NAME/module.modulemap || true
    cp target/uniffi-xcframework-staging-utils/${ORIG_FFI_MODULE_NAME}.h target/uniffi-xcframework-staging-utils/simulator/Headers/$UTILS_FFI_MODULE_NAME/${UTILS_FFI_MODULE_NAME}.h || true
  fi

  # Use renamed static library filenames to avoid collisions with Core
  local device_lib_original="target/aarch64-apple-ios/$PROFILE/lib$1.a"
  local device_lib_renamed="target/aarch64-apple-ios/$PROFILE/lib$1-utils.a"
  local sim_lib_original="$fat_simulator_lib_dir/lib$1.a"
  local sim_lib_renamed="$fat_simulator_lib_dir/lib$1-utils.a"
  cp "$device_lib_original" "$device_lib_renamed"
  cp "$sim_lib_original" "$sim_lib_renamed"

  xcodebuild -create-xcframework \
      -library "$device_lib_renamed" -headers target/uniffi-xcframework-staging-utils/device/Headers \
      -library "$sim_lib_renamed" -headers target/uniffi-xcframework-staging-utils/simulator/Headers \
      -output "target/ios-utils/lib$1-utils.xcframework"
}

create_fat_simulator_lib() {
  echo "Creating a fat library for x86_64 and aarch64 simulators (Utils)..."
  mkdir -p "$fat_simulator_lib_dir"
  lipo -create \
      "target/x86_64-apple-ios/$PROFILE/lib$1.a" \
      "target/aarch64-apple-ios-sim/$PROFILE/lib$1.a" \
      -output "$fat_simulator_lib_dir/lib$1.a"
}

copy_swift_sources() {
  echo "Copying Swift source files to YttriumUtils package..."
  # Ensure the Swift package sources directory exists
  mkdir -p platforms/swift/Sources/YttriumUtils
  
  # Copy the generated Swift file
  cp target/uniffi-xcframework-staging-utils/yttrium.swift platforms/swift/Sources/YttriumUtils/

  # Namespace the import to avoid clashing with Core
  sed -i '' "s/canImport($ORIG_FFI_MODULE_NAME)/canImport($UTILS_FFI_MODULE_NAME)/" platforms/swift/Sources/YttriumUtils/yttrium.swift
  sed -i '' "s/import $ORIG_FFI_MODULE_NAME/import $UTILS_FFI_MODULE_NAME/" platforms/swift/Sources/YttriumUtils/yttrium.swift
}

# Add the nightly toolchain with rust-src component (required for -Z build-std)
rustup toolchain install nightly --component rust-src
rustup target add aarch64-apple-ios --toolchain nightly
rustup target add x86_64-apple-ios --toolchain nightly
rustup target add aarch64-apple-ios-sim --toolchain nightly

# Execute the build steps
build_rust_libraries
generate_ffi $PACKAGE_NAME
rename_ffi_module
create_fat_simulator_lib $PACKAGE_NAME
build_xcframework $PACKAGE_NAME
copy_swift_sources 
