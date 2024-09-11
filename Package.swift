// swift-tools-version: 5.7
import PackageDescription

let package = Package(
    name: "yttrium",
    platforms: [
        .macOS(.v13),
        .iOS(.v13),
        .watchOS(.v9),
        .tvOS(.v16)
    ],
    products: [
        .library(
            name: "Yttrium",
            targets: ["Yttrium"]),
    ],
    dependencies: [
        .package(path: "crates/ffi/YttriumCore"),
        .package(url: "https://github.com/thebarndog/swift-dotenv.git", from: "2.0.0")
    ],
    targets: [
        .target(
            name: "Yttrium",
            dependencies: [
                "YttriumCore",
                .product(name: "SwiftDotenv", package: "swift-dotenv")
            ],
            path: "platforms/swift/Sources/Yttrium")
        ,
        .testTarget(
            name: "YttriumTests",
            dependencies: [
                "Yttrium" ,
                .product(name: "SwiftDotenv", package: "swift-dotenv")
            ],
            path: "platforms/swift/Tests/YttriumTests"),
    ]
)
