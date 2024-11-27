#!/bin/bash

set -e  # Exit on error

PACKAGE_NAME="uniffi_yttrium"  # Must match [lib] name in Cargo.toml
STAGING_DIR="target/uniffi-xcframework-staging"
XCFRAMEWORK_DIR="target/ios"
FAT_SIMULATOR_LIB_DIR="target/ios-simulator-fat/release"
SWIFT_PACKAGE_DIR="platforms/swift/Sources/Yttrium"

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
    --crate "$PACKAGE_NAME" \
    --out-dir "$STAGING_DIR"

# Ensure modulemap is correctly named
if [ -f "$STAGING_DIR/$PACKAGE_NAME.modulemap" ]; then
    mv "$STAGING_DIR/$PACKAGE_NAME.modulemap" "$STAGING_DIR/module.modulemap"
elif [ -f "$STAGING_DIR/${PACKAGE_NAME}FFI.modulemap" ]; then
    mv "$STAGING_DIR/${PACKAGE_NAME}FFI.modulemap" "$STAGING_DIR/module.modulemap"
else
    echo "No modulemap found to rename!"
    exit 1
fi

# 4. Create XCFramework
echo "Creating XCFramework..."
rm -rf "$XCFRAMEWORK_DIR"  # Clean XCFramework directory
mkdir -p "$XCFRAMEWORK_DIR"

xcodebuild -create-xcframework \
    -library target/aarch64-apple-ios/release/lib$PACKAGE_NAME.a -headers "$STAGING_DIR" \
    -library "$FAT_SIMULATOR_LIB_DIR/lib$PACKAGE_NAME.a" -headers "$STAGING_DIR" \
    -output "$XCFRAMEWORK_DIR/lib$PACKAGE_NAME.xcframework"

# 5. Copy outputs to Swift Package Directory
echo "Copying Swift bindings and runtime files to Swift package..."
rm -rf "$SWIFT_PACKAGE_DIR"  # Clean Swift package directory
mkdir -p "$SWIFT_PACKAGE_DIR"

cp "$STAGING_DIR"/*.swift "$SWIFT_PACKAGE_DIR/"
cp "$STAGING_DIR"/*.h "$SWIFT_PACKAGE_DIR/"
cp "$STAGING_DIR/module.modulemap" "$SWIFT_PACKAGE_DIR/" || echo "No modulemap to copy."

echo "Copying XCFramework to Swift package..."
rm -rf "$SWIFT_PACKAGE_DIR/lib$PACKAGE_NAME.xcframework"
cp -R "$XCFRAMEWORK_DIR/lib$PACKAGE_NAME.xcframework" "$SWIFT_PACKAGE_DIR/"

echo "Build and setup completed."
