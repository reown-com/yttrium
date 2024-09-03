# build-rust.sh

#!/bin/bash

PACKAGE_NAME=ffi
SWIFT_PACKAGE_NAME=YttriumCore

set -e

THISDIR=$(dirname $0)
cd $THISDIR

echo "Building for macOS..."

rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

cargo build --target x86_64-apple-darwin
cargo build --target aarch64-apple-darwin

mkdir -p ./../../target/universal-macos/debug

echo "Lipoing for macOS..."

lipo \
    ./../../target/aarch64-apple-darwin/debug/lib$PACKAGE_NAME.a \
    ./../../target/x86_64-apple-darwin/debug/lib$PACKAGE_NAME.a -create -output \
    ./../../target/universal-macos/debug/lib$PACKAGE_NAME.a

echo "Building for iOS..."

rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios
rustup target add aarch64-apple-ios-sim

cargo build --target aarch64-apple-ios
cargo build --target x86_64-apple-ios
cargo build --target aarch64-apple-ios-sim

mkdir -p ./../../target/universal-ios/debug

echo "Lipoing for iOS..."

lipo \
    ./../../target/aarch64-apple-ios-sim/debug/lib$PACKAGE_NAME.a \
    ./../../target/x86_64-apple-ios/debug/lib$PACKAGE_NAME.a -create -output \
    ./../../target/universal-ios/debug/lib$PACKAGE_NAME.a

# function create_package (); 

function create_package {
  cargo install -f swift-bridge-cli
  swift-bridge-cli create-package \
        --bridges-dir ./generated \
        --out-dir $SWIFT_PACKAGE_NAME \
        --ios ./../../target/aarch64-apple-ios/debug/lib$PACKAGE_NAME.a \
        --simulator ./../../target/universal-ios/debug/lib$PACKAGE_NAME.a \
        --macos ./../../target/universal-macos/debug/lib$PACKAGE_NAME.a \
        --name $SWIFT_PACKAGE_NAME
}

# Check if Package.swift file exists
if [ -f $SWIFT_PACKAGE_NAME/Package.swift ]; then
    echo "Package.swift already exists. Copying existing file to backup..."
    rm -f $SWIFT_PACKAGE_NAME/Package.swift.bak
    cp $SWIFT_PACKAGE_NAME/Package.swift $SWIFT_PACKAGE_NAME/Package.swift.bak
    echo "Creating Swift package..."
    create_package
    cp $SWIFT_PACKAGE_NAME/Package.swift.bak $SWIFT_PACKAGE_NAME/Package.swift
    rm -f $SWIFT_PACKAGE_NAME/Package.swift.bak
else
    echo "Creating Swift package..."
    create_package
fi
