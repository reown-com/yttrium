#!/bin/bash

set -e  # Exit on error

PACKAGE_NAME="uniffi_yttrium"  # Must match [lib] name in Cargo.toml
STAGING_DIR="target/uniffi-xcframework-staging"
XCFRAMEWORK_DIR="target/ios"
FAT_SIMULATOR_LIB_DIR="target/ios-simulator-fat/release"

# 1. Build Rust libraries
echo "Building Rust libraries for iOS targets..."
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim

#cargo build --release --target aarch64-apple-ios
#cargo build --release --target x86_64-apple-ios
#cargo build --release --target aarch64-apple-ios-sim

# 2. Create Fat Simulator Library
echo "Creating fat library for iOS simulators..."
mkdir -p "$FAT_SIMULATOR_LIB_DIR"
lipo -create \
    target/x86_64-apple-ios/release/lib$PACKAGE_NAME.a \
    target/aarch64-apple-ios-sim/release/lib$PACKAGE_NAME.a \
    -output "$FAT_SIMULATOR_LIB_DIR/lib$PACKAGE_NAME.a"

# 3. Generate FFI Bindings with UniFFI
echo "Generating FFI bindings with UniFFI..."
rm -rf "$STAGING_DIR"  # Clean staging directory
mkdir -p "$STAGING_DIR"

cargo run --features uniffi/cli --bin uniffi-bindgen generate \
    --library target/aarch64-apple-ios/release/lib$PACKAGE_NAME.dylib \
    --language swift \
    --out-dir "$STAGING_DIR"

# Ensure modulemap is correctly named
mv "$STAGING_DIR/$PACKAGE_NAME.modulemap" "$STAGING_DIR/module.modulemap" || true

# 4. Create XCFramework
echo "Creating XCFramework..."
rm -rf "$XCFRAMEWORK_DIR"  # Clean XCFramework directory
mkdir -p "$XCFRAMEWORK_DIR"

xcodebuild -create-xcframework \
    -library target/aarch64-apple-ios/release/lib$PACKAGE_NAME.a -headers "$STAGING_DIR" \
    -library "$FAT_SIMULATOR_LIB_DIR/lib$PACKAGE_NAME.a" -headers "$STAGING_DIR" \
    -output "$XCFRAMEWORK_DIR/lib$PACKAGE_NAME.xcframework"

# 5. Copy outputs to Swift Package Directory
SWIFT_PACKAGE_DIR="platforms/swift/Sources/Yttrium"

echo "Copying Swift bindings to Swift package..."
mkdir -p "$SWIFT_PACKAGE_DIR"
cp "$STAGING_DIR"/*.swift "$SWIFT_PACKAGE_DIR/"

echo "Copying XCFramework to Swift package..."
cp -R "$XCFRAMEWORK_DIR/lib$PACKAGE_NAME.xcframework" "$SWIFT_PACKAGE_DIR/"

echo "Build and setup completed."
