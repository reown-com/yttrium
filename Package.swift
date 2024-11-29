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
            url: "https://github.com/reown-com/yttrium/releases/download/0.2.20/libuniffi_yttrium.xcframework.zip",
            checksum: "cf36bb36fd2adffffa2857e497bc29ed6b1e976ac1ce19b2124f3a8a4843f76b"
        )
    ]
)

