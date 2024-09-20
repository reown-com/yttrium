// swift-tools-version: 5.10
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
        .library(
            name: "YttriumDev",
            targets: ["YttriumDev"]
        ),
    ],
    dependencies: [
        .package(url: "https://github.com/thebarndog/swift-dotenv.git", from: "2.0.0")
    ],
    targets: [
        .target(
            name: "Yttrium",
            dependencies: [
                "YttriumCore",
                .product(name: "SwiftDotenv", package: "swift-dotenv")
            ],
            path: "platforms/swift/Sources/Yttrium"
        ),
        .target(
            name: "YttriumDev",
            dependencies: [
                "YttriumCoreDev",
                .product(name: "SwiftDotenv", package: "swift-dotenv")
            ],
            path: "platforms/swift/Sources/Yttrium"
        ),
        .target(
            name: "YttriumCore",
            dependencies: [
                "RustXcframeworkRelease"
            ],
            path: "crates/ffi/YttriumCore/Sources/YttriumCore"
        ),
        .target(
            name: "YttriumCoreDev",
            dependencies: [
                "RustXcframeworkDev"
            ],
            path: "crates/ffi/YttriumCore/Sources/YttriumCore"
        ),
        .binaryTarget(
            name: "RustXcframeworkRelease",
            url: "https://github.com/WalletConnect/yttrium/releases/download/0.0.1-alpha-test/RustXcframework.xcframework.zip",
            checksum: "360ecd0ec70ec79883e03969cf359528e5de9c021a0177450506729eb523474f"
        ),
        .binaryTarget(
            name: "RustXcframeworkDev",
            path: "crates/ffi/YttriumCore/RustXcframework.xcframework"
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
