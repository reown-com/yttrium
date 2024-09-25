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
            url: "https://github.com/reown-com/yttrium/releases/download/0.0.20/RustXcframework.xcframework.zip",
            checksum: "9fb2dc7191822798d441b016a2c460ecebefa97e08494285d530585e522b9b67"
        ),
        .target(
            name: "YttriumCore",
            dependencies: [
                "RustXcframework"
            ],
            path: "crates/ffi/YttriumCore/Sources/YttriumCore"
        ),
        .target(
            name: "Yttrium",
            dependencies: [
                "YttriumCore",
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
