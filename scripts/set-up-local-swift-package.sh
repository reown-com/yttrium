#!/bin/bash
set -e

# Build both Core and Utils XCFrameworks locally
make build-xcframework
make build-utils-xcframework

# Write a local Package.swift that references the locally built XCFrameworks
cat > Package.swift <<'SWIFT'
// swift-tools-version:5.10
import PackageDescription
import Foundation

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
        .library(
            name: "YttriumUtils",
            targets: ["YttriumUtils"]
        ),
    ],
    targets: [
        .target(
            name: "Yttrium",
            dependencies: ["YttriumXCFramework"],
            path: "platforms/swift/Sources/Yttrium"
        ),
        .target(
            name: "YttriumUtils",
            dependencies: ["YttriumUtilsXCFramework"],
            path: "platforms/swift/Sources/YttriumUtils"
        ),
        .binaryTarget(
            name: "YttriumXCFramework",
            path: "target/ios/libyttrium.xcframework"
        ),
        .binaryTarget(
            name: "YttriumUtilsXCFramework",
            path: "target/ios-utils/libyttrium-utils.xcframework"
        )
    ]
)
SWIFT

# Prevent accidental commits of local Package.swift
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  git update-index --assume-unchanged Package.swift || true
fi

echo "Local Package.swift has been written and both XCFrameworks built."
echo "Package.swift marked as assume-unchanged to avoid committing local paths."
echo "To revert: git update-index --no-assume-unchanged Package.swift && git checkout -- Package.swift"

