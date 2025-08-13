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
        url: "https://github.com/reown-com/yttrium/releases/download/0.9.67/libyttrium.xcframework.zip",
        checksum: "877203641579d0f06d031ae3e05057050431c663874430e278e9c89ab7ca75f4"
    )

let yttriumUtilsXcframeworkTarget: Target = useLocalRustXcframework ?
    .binaryTarget(
        name: "YttriumUtilsXCFramework",
        path: "target/ios-utils/libyttrium-utils.xcframework"
    ) :
    .binaryTarget(
        name: "YttriumUtilsXCFramework",
        url: "https://github.com/reown-com/yttrium/releases/download/0.9.64/libyttrium-utils.xcframework.zip",
        checksum: "4c344ef8463f39dce1e5c7e0d58bd2b27be2cfb882e33ff48b8810858f045d37"
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
