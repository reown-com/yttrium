// swift-tools-version:5.10
import PackageDescription
import Foundation

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
        .target(
            name: "Yttrium",
            dependencies: ["YttriumXCFramework"],
            path: "platforms/swift/Sources/Yttrium",
            publicHeadersPath: ".",
            cSettings: [
                .headerSearchPath("."),
                // Tells the compiler to use yttrium.modulemap instead of the default module.modulemap
                .unsafeFlags(["-fmodule-map-file=yttrium.modulemap"])
            ]
        ),
        .binaryTarget(
            name: "YttriumXCFramework",
            path: "target/ios/libuniffi_yttrium.xcframework"
        )
    ]
)
