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
        url: "https://github.com/reown-com/yttrium/releases/download/0.9.111/libyttrium.xcframework.zip",
        checksum: "51f1f38e0e9278aeb946d7cdfa9a971e412f0e485b23a1e9674fecb8593e9bc4"
    )

let yttriumUtilsXcframeworkTarget: Target = useLocalRustXcframework ?
    .binaryTarget(
        name: "YttriumUtilsXCFramework",
        path: "target/ios-utils/libyttrium-utils.xcframework"
    ) :
    .binaryTarget(
        name: "YttriumUtilsXCFramework",
        url: "https://github.com/reown-com/yttrium/releases/download/0.9.80/libyttrium-utils.xcframework.zip",
        checksum: "b8b08dfe910a6589bb47c43ab5cb8e506d0f2ca841f602853706c18c9e1af445"
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
