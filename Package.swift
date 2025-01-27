// swift-tools-version:5.10
import PackageDescription
import Foundation

let useLocalRustXcframework = ProcessInfo.processInfo.environment["USE_LOCAL_RUST_XCFRAMEWORK"] == "1"

let yttriumXcframeworkTarget: Target = useLocalRustXcframework ?
    .binaryTarget(
        name: "YttriumXCFramework",
        path: "target/ios/libuniffi_yttrium.xcframework"
    ) :
    .binaryTarget(
        name: "YttriumXCFramework",
        url: "https://github.com/reown-com/yttrium/releases/download/0.6.0/libuniffi_yttrium.xcframework.zip",
        checksum: "0809574bba760d69befc6854463fb53b5ea9540fc57e519c24a540ca24b9e0fd"
    )

let package = Package(
    name: "Yttrium",
    platforms: [
        .iOS(.v13), .macOS(.v12)
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
