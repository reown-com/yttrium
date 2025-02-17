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
        url: "https://github.com/reown-com/yttrium/releases/download/0.8.21/libuniffi_yttrium.xcframework.zip",
        checksum: "e616d5a89870703f804319514d1fe227c1d9039692d2eef747dc27d537e6b628"
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
