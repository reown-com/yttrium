#!/usr/bin/env zsh

set -e
set -u

PACKAGE_NAME=uniffi_yttrium  # Must match the [lib] name in Cargo.toml
fat_simulator_lib_dir="target/ios-simulator-fat/release"

# Build Rust libraries
echo "Building Rust library for iOS targets..."
cargo build -p $PACKAGE_NAME --lib --release --target x86_64-apple-ios
cargo build -p $PACKAGE_NAME --lib --release --target aarch64-apple-ios-sim
cargo build -p $PACKAGE_NAME --lib --release --target aarch64-apple-ios

generate_ffi() {
  echo "Generating FFI bindings with UniFFI..."

  # Use the `uniffi-bindgen` binary explicitly
  cargo run --features uniffi/cli --bin uniffi-bindgen generate \
    --library target/aarch64-apple-ios/release/lib$1.dylib \
    --language swift \
    --out-dir target/uniffi-xcframework-staging

  # Ensure the modulemap is named correctly
  mv target/uniffi-xcframework-staging/$1FFI.modulemap target/uniffi-xcframework-staging/module.modulemap || true

  echo "FFI bindings generated successfully."
}

create_fat_simulator_lib() {
  echo "Creating a fat library for x86_64 and aarch64 simulators..."
  mkdir -p $fat_simulator_lib_dir
  lipo -create target/x86_64-apple-ios/release/lib$1.a target/aarch64-apple-ios-sim/release/lib$1.a -output $fat_simulator_lib_dir/lib$1.a
}

build_xcframework() {
  echo "Generating XCFramework..."
  rm -rf target/ios  # Delete the output folder so we can regenerate it
  xcodebuild -create-xcframework \
    -library target/aarch64-apple-ios/release/lib$1.a -headers target/uniffi-xcframework-staging \
    -library target/ios-simulator-fat/release/lib$1.a -headers target/uniffi-xcframework-staging \
    -output target/ios/lib$1-rs.xcframework
}

generate_ffi $PACKAGE_NAME
create_fat_simulator_lib $PACKAGE_NAME
build_xcframework $PACKAGE_NAME
