// swift-tools-version:5.9.0
import PackageDescription
let package = Package(
	name: "YttriumCore",
	platforms: [
        .macOS(.v14),
        .iOS(.v13),
        .watchOS(.v10),
        .tvOS(.v17)
    ],
	products: [
		.library(
			name: "YttriumCore",
			targets: ["YttriumCore"]
		),
	],
	dependencies: [
		.package(url: "https://github.com/thebarndog/swift-dotenv.git", from: "2.0.0")
	],
	targets: [
		.binaryTarget(
			name: "RustXcframework",
			path: "RustXcframework.xcframework"
		),
		.target(
			name: "YttriumCore",
			dependencies: [
                "RustXcframework",
				.product(name: "SwiftDotenv", package: "swift-dotenv")
            ]
        ),
        .testTarget(
            name: "YttriumCoreTests",
            dependencies: [
				"YttriumCore"
			]
		),
	]
)
