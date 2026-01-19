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
        url: "https://github.com/reown-com/yttrium/releases/download/0.10.10/libyttrium.xcframework.zip",
        checksum: "39c4f247809410de13a5322bfe8bed80be74d8d328b9a156b6025877bd02c5ad"
    )

let yttriumUtilsXcframeworkTarget: Target = useLocalRustXcframework ?
    .binaryTarget(
        name: "YttriumUtilsXCFramework",
        path: "target/ios-utils/libyttrium-utils.xcframework"
    ) :
    .binaryTarget(
        name: "YttriumUtilsXCFramework",
        url: "https://github.com/reown-com/yttrium/releases/download/0.10.1/libyttrium-utils.xcframework.zip",
        checksum: "8a84ecd824b146121ff17a617ffc41f7649d8023f976fae6a3a95b08bbc784b6"
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
