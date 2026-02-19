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
        url: "https://github.com/reown-com/yttrium/releases/download/0.10.40/libyttrium.xcframework.zip",
        checksum: "fe9faa0a26b09e09e6c914994dc1bed850f2cb5d62ffd7a9be69589f565edc9d"
    )

let yttriumUtilsXcframeworkTarget: Target = useLocalRustXcframework ?
    .binaryTarget(
        name: "YttriumUtilsXCFramework",
        path: "target/ios-utils/libyttrium-utils.xcframework"
    ) :
    .binaryTarget(
        name: "YttriumUtilsXCFramework",
        url: "https://github.com/reown-com/yttrium/releases/download/0.10.42/libyttrium-utils.xcframework.zip",
        checksum: "b90b1b03af45c253d6f2bf26ba48e5b33a9997fd620d709c5e2b1e96c931d588"
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
