#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

PACKAGE_NAME=uniffi_yttrium  # Must match the [lib] name in Cargo.toml
SWIFT_PACKAGE_DIR="platforms/swift/Sources/Yttrium"

echo "Building for iOS devices and simulators..."

# Add required Rust targets for iOS
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim

## Build for iOS devices
#cargo build --release --target aarch64-apple-ios
#
## Build for iOS simulators (Intel and ARM)
#cargo build --release --target x86_64-apple-ios
#cargo build --release --target aarch64-apple-ios-sim

# Create a universal library for iOS simulators using lipo
echo "Creating universal library for iOS simulators with lipo..."
mkdir -p target/universal-ios/release
lipo \
    target/aarch64-apple-ios-sim/release/lib$PACKAGE_NAME.a \
    target/x86_64-apple-ios/release/lib$PACKAGE_NAME.a -create -output \
    target/universal-ios/release/lib$PACKAGE_NAME.a

# Generate Swift bindings using UniFFI
echo "Generating Swift bindings with UniFFI..."
cargo run --features uniffi/cli --bin uniffi-bindgen generate \
    --library target/aarch64-apple-ios/release/lib$PACKAGE_NAME.dylib \
    --language swift \
    --out-dir ./generated

# Remove existing XCFramework if it exists
echo "Removing existing xcframework..."
rm -rf ./generated/$PACKAGE_NAME.xcframework

# Create the XCFramework
echo "Creating new xcframework..."
xcodebuild -create-xcframework \
    -library target/aarch64-apple-ios/release/lib$PACKAGE_NAME.a \
    -headers ./generated \
    -library target/universal-ios/release/lib$PACKAGE_NAME.a \
    -headers ./generated \
    -output ./generated/$PACKAGE_NAME.xcframework

# Copy the generated Swift bindings to the Swift package source directory
echo "Copying generated Swift bindings to Swift package..."
mkdir -p $SWIFT_PACKAGE_DIR
cp ./generated/*.swift $SWIFT_PACKAGE_DIR/

# Copy the XCFramework to the Swift package
echo "Copying XCFramework to Swift package..."
cp -R ./generated/$PACKAGE_NAME.xcframework $SWIFT_PACKAGE_DIR/

echo "Build and setup completed."
