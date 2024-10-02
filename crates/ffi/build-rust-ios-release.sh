# build-rust-ios.sh

#!/bin/bash

PACKAGE_NAME=ffi
SWIFT_PACKAGE_NAME=YttriumCore

set -e

THISDIR=$(dirname $0)
cd $THISDIR

echo "Building for iOS..."

rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios
rustup target add aarch64-apple-ios-sim

cargo build --release --target aarch64-apple-ios
cargo build --release --target x86_64-apple-ios
cargo build --release --target aarch64-apple-ios-sim

mkdir -p ./../../target/universal-ios/release

echo "Lipoing for iOS..."

lipo \
    ./../../target/aarch64-apple-ios-sim/release/lib$PACKAGE_NAME.a \
    ./../../target/x86_64-apple-ios/release/lib$PACKAGE_NAME.a -create -output \
    ./../../target/universal-ios/release/lib$PACKAGE_NAME.a

function create_package {
  cargo install -f swift-bridge-cli
  swift-bridge-cli create-package \
        --bridges-dir ./generated \
        --out-dir $SWIFT_PACKAGE_NAME \
        --ios ./../../target/aarch64-apple-ios/release/lib$PACKAGE_NAME.a \
        --simulator ./../../target/universal-ios/release/lib$PACKAGE_NAME.a \
        --name $SWIFT_PACKAGE_NAME

    # Make generated methods public in the ffi.swift file
    sed -i '' 's/^func __swift_bridge__/public func __swift_bridge__/g' YttriumCore/Sources/YttriumCore/ffi.swift
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
