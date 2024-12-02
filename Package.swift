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
            url: "https://github.com/reown-com/yttrium/releases/download/02.21/libuniffi_yttrium.xcframework.zip",
            checksum: "fc585014bbda3a054532bb04d3c182f6c6ea24c70ca03ad53c1e030867ce8632"
        )
    ]
)

