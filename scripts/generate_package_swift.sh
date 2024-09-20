#!/bin/bash

set -e

# Variables
PACKAGE_VERSION="${GITHUB_VERSION:-0.0.1-alpha}-test"
RUST_CHECKSUM=$(cat rust_checksum.txt)
RUST_XCFRAMEWORK_ZIP="RustXcframework.xcframework.zip"
REPO_URL="https://github.com/WalletConnect/yttrium"

# Generate Package.swift
cat > Package.swift <<EOF
// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "yttrium",
    platforms: [
        .macOS(.v14),
        .iOS(.v13),
        .watchOS(.v10),
        .tvOS(.v17)
    ],
    products: [
        .library(
            name: "Yttrium",
            targets: ["Yttrium"]
        ),
        .library(
            name: "YttriumDev",
            targets: ["YttriumDev"]
        ),
    ],
    dependencies: [
        .package(url: "https://github.com/thebarndog/swift-dotenv.git", from: "2.0.0")
    ],
    targets: [
        .target(
            name: "Yttrium",
            dependencies: [
                "YttriumCore",
                .product(name: "SwiftDotenv", package: "swift-dotenv")
            ],
            path: "platforms/swift/Sources/Yttrium"
        ),
        .target(
            name: "YttriumDev",
            dependencies: [
                "YttriumCoreDev",
                .product(name: "SwiftDotenv", package: "swift-dotenv")
            ],
            path: "platforms/swift/Sources/Yttrium"
        ),
        .target(
            name: "YttriumCore",
            dependencies: [
                "RustXcframeworkRelease"
            ],
            path: "crates/ffi/YttriumCore/Sources/YttriumCore"
        ),
        .target(
            name: "YttriumCoreDev",
            dependencies: [
                "RustXcframeworkDev"
            ],
            path: "crates/ffi/YttriumCore/Sources/YttriumCore"
        ),
        .binaryTarget(
            name: "RustXcframeworkRelease",
            url: "$REPO_URL/releases/download/$PACKAGE_VERSION/$RUST_XCFRAMEWORK_ZIP",
            checksum: "$RUST_CHECKSUM"
        ),
        .binaryTarget(
            name: "RustXcframeworkDev",
            path: "crates/ffi/YttriumCore/RustXcframework.xcframework"
        ),
        .testTarget(
            name: "YttriumTests",
            dependencies: [
                "Yttrium",
                .product(name: "SwiftDotenv", package: "swift-dotenv")
            ],
            path: "platforms/swift/Tests/YttriumTests"
        ),
    ]
)
EOF

echo "Package.swift generated with Rust XCFramework checksum: $RUST_CHECKSUM"