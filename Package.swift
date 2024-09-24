// swift-tools-version:5.10
import PackageDescription

let package = Package(
    name: "yttrium",
    platforms: [
        .macOS(.v14),
        .iOS(.v13),
        .watchOS(.v10),
        .tvOS(.v17)
    ],
    products: [
        .library(
            name: "Yttrium",
            targets: ["Yttrium"]
        ),
    ],
    dependencies: [
        // Other dependencies if any
    ],
    targets: [
        .binaryTarget(
            name: "RustXcframework",
            url: "https://github.com/reown-com/yttrium/releases/download/0.0.8/RustXcframework.xcframework.zip",
            checksum: "2a9b8823a8b6e02d4590a24e3ce0f56184e88c14ec4d6f64e0a39c57c2efb3b1"
        ),
        .target(
            name: "YttriumCore",
            dependencies: [
                "RustXcframework"
            ],
            path: "crates/ffi",
            exclude: ["YttriumCore/RustXcframework.xcframework"],
            sources: [
                "YttriumCore/Sources/YttriumCore",
                "generated"
            ]
        ),
        .target(
            name: "Yttrium",
            dependencies: [
                "YttriumCore"
            ],
            path: "platforms/swift/Sources/Yttrium"
        ),
        .testTarget(
            name: "YttriumTests",
            dependencies: [
                "Yttrium"
            ],
            path: "platforms/swift/Tests/YttriumTests"
        ),
    ]
)