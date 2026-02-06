// swift-tools-version:5.10
import PackageDescription
import Foundation

let useLocalRustXcframework = ProcessInfo.processInfo.environment["USE_LOCAL_RUST_XCFRAMEWORK"] == "1"

let yttriumXcframeworkTarget: Target = useLocalRustXcframework ?
    .binaryTarget(
        name: "YttriumXCFramework",
        path: "target/ios/libyttrium.xcframework"
    ) :
    .binaryTarget(
        name: "YttriumXCFramework",
        url: "https://github.com/reown-com/yttrium/releases/download/0.10.33/libyttrium.xcframework.zip",
        checksum: "0c32116d94656803ff2a53b6fe866f4f10721a9bc78182f27d030075f5f3254b"
    )

let yttriumUtilsXcframeworkTarget: Target = useLocalRustXcframework ?
    .binaryTarget(
        name: "YttriumUtilsXCFramework",
        path: "target/ios-utils/libyttrium-utils.xcframework"
    ) :
    .binaryTarget(
        name: "YttriumUtilsXCFramework",
        url: "https://github.com/reown-com/yttrium/releases/download/0.10.33/libyttrium-utils.xcframework.zip",
        checksum: "a4cf5f2502b0bace1d45c28c06cfc8a731166234e7a497c5fd89ed9f0edc2aed"
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
        .library(
            name: "YttriumUtils",
            targets: ["YttriumUtils"]
        ),
    ],
    targets: [
        yttriumXcframeworkTarget,
        yttriumUtilsXcframeworkTarget,
        .target(
            name: "Yttrium",
            dependencies: ["YttriumXCFramework"],
            path: "platforms/swift/Sources/Yttrium",
            publicHeadersPath: ".",
            cSettings: [
                .headerSearchPath(".")
            ]
        ),
        .target(
            name: "YttriumUtils",
            dependencies: ["YttriumUtilsXCFramework"],
            path: "platforms/swift/Sources/YttriumUtils",
            publicHeadersPath: ".",
            cSettings: [
                .headerSearchPath(".")
            ]
        )
    ]
)
