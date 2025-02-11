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
        url: "https://github.com/reown-com/yttrium/releases/download/0.8.13/libuniffi_yttrium.xcframework.zip",
        checksum: "44bd85174fee3fbb9d7f453162cf1746c3b76ddfa33b1d9a620c0cea8b4453cc"
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
