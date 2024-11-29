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
            url: "https://github.com/reown-com/yttrium/releases/download/0.2.18/libuniffi_yttrium.xcframework.zip",
            checksum: "ef232fafe26bdbc050515c198c42b12d2fba0f737afbe3b43eaa70f87a1ab6ab"
        )
    ]
)

