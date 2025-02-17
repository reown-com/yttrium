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
                .headerSearchPath(".")
            ]
        ),
        .binaryTarget(
            name: "YttriumXCFramework",
            path: "target/ios/libuniffi_yttrium.xcframework"
        )
    ]
)
