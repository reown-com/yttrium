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
        url: "https://github.com/reown-com/yttrium/releases/download/0.0.22/RustXcframework.xcframework.zip",
        checksum: "aadc4f0cc497950d87d5c8de5a45889815d44a5437a19ba3b6600f206549671c"
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