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
        .package(url: "https://github.com/thebarndog/swift-dotenv.git", from: "2.0.0")
    ],
    targets: [
        .binaryTarget(
            name: "RustXcframework",
            url: "https://github.com/reown-com/yttrium/releases/download/0.0.7/RustXcframework.xcframework.zip",
            checksum: "67c654c20259846d3597abafb378993e49782f3636a578c5984b8a9a28112943"
        ),
        .target(
            name: "Yttrium",
            dependencies: [
                "RustXcframework",
                .product(name: "SwiftDotenv", package: "swift-dotenv")
            ],
            path: "platforms/swift/Sources/Yttrium"
        ),
        .testTarget(
            name: "YttriumTests",
            dependencies: [
                "Yttrium",
                .product(name: "SwiftDotenv", package: "swift-dotenv")
            ],
            path: "platforms/swift/Tests/YttriumTests"
        ),
    ]
)
