#!/bin/bash

set -e
set -u

# Variables
: "${VERSION:?Error: VERSION environment variable is not set.}"
PACKAGE_VERSION="$VERSION"
RUST_XCFRAMEWORK_DIR="target/ios/libyttrium.xcframework"
RUST_XCFRAMEWORK_ZIP="libyttrium.xcframework.zip"
REPO_URL="https://github.com/reown-com/yttrium"

# 1. Zip the XCFramework
echo "Zipping Rust XCFramework..."
mkdir -p Output
zip -r Output/$RUST_XCFRAMEWORK_ZIP $RUST_XCFRAMEWORK_DIR

# 2. Compute the checksum
echo "Computing checksum for Rust XCFramework..."
RUST_CHECKSUM=$(swift package compute-checksum Output/$RUST_XCFRAMEWORK_ZIP)
echo "Rust XCFramework checksum: $RUST_CHECKSUM"

# 3. Generate Package.swift
# If USE_LOCAL_RUST_XCFRAMEWORK=1 is set in the environment, it will use the local xcframework path
# Otherwise, it uses the remote URL and checksum.
echo "Generating Package.swift..."
cat > Package.swift <<EOF
// swift-tools-version:5.10
import PackageDescription
import Foundation

let useLocalRustXcframework = ProcessInfo.processInfo.environment["USE_LOCAL_RUST_XCFRAMEWORK"] == "1"

let yttriumXcframeworkTarget: Target = useLocalRustXcframework ?
    .binaryTarget(
        name: "YttriumXCFramework",
        path: "$RUST_XCFRAMEWORK_DIR"
    ) :
    .binaryTarget(
        name: "YttriumXCFramework",
        url: "$REPO_URL/releases/download/$PACKAGE_VERSION/$RUST_XCFRAMEWORK_ZIP",
        checksum: "$RUST_CHECKSUM"
    )

let package = Package(
    name: "Yttrium",
    platforms: [
        .iOS(.v13), .macOS(.v11)
    ],
    products: [
        .library(
            name: "Yttrium",
            targets: ["Yttrium"]
        ),
    ],
    targets: [
        yttriumXcframeworkTarget,
        .target(
            name: "Yttrium",
            dependencies: ["YttriumXCFramework"],
            path: "platforms/swift/Sources/Yttrium",
            publicHeadersPath: ".",
            cSettings: [
                .headerSearchPath(".")
            ]
        )
    ]
)
EOF

echo "Package.swift generated with Rust XCFramework checksum: $RUST_CHECKSUM"
