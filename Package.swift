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
        url: "https://github.com/reown-com/yttrium/releases/download/0.1.0/RustXcframework.xcframework.zip",
        checksum: "28ccb544483685ef2dc27dec3ba2bf193caabbe4bdae4cf7ae65a2b5b7b95bd2"
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
    ],
    targets: [
        rustXcframeworkTarget,
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
