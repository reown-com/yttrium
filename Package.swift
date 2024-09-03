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
            targets: ["Yttrium"]),
    ],
    dependencies: [
        .package(path: "crates/ffi/YttriumCore")
    ],
    targets: [
        .target(
            name: "Yttrium",
            dependencies: [
                "YttriumCore"
            ],
            path: "platforms/swift/Sources/Yttrium")
        ,
        .testTarget(
            name: "YttriumTests",
            dependencies: ["Yttrium"],
            path: "platforms/swift/Tests/YttriumTests"),
    ]
)
