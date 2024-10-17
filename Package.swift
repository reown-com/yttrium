// swift-tools-version:5.10
import PackageDescription
import Foundation

let useLocalRustXcframework = ProcessInfo.processInfo.environment["USE_LOCAL_RUST_XCFRAMEWORK"] == "1"

let rustXcframeworkTarget: Target = useLocalRustXcframework ?
    .binaryTarget(
        name: "RustXcframework",
        path: "crates/ffi/YttriumCore/RustXcframework.xcframework"
    ) :
    .binaryTarget(
        name: "RustXcframework",
        url: "https://github.com/reown-com/yttrium/releases/download/0.2.0/RustXcframework.xcframework.zip",
        checksum: "7b3381f2938acbc099867b88108f9cabcfb9f8c0b535e0b9262d4e92c6eb893c"
    )

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
        rustXcframeworkTarget,
        .target(
            name: "YttriumCore",
            dependencies: [
                "RustXcframework",
                .product(name: "SwiftDotenv", package: "swift-dotenv")
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
