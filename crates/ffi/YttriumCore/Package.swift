// swift-tools-version: 5.7
import PackageDescription
let package = Package(
	name: "YttriumCore",
	platforms: [
        .macOS(.v13),
        .iOS(.v13),
        .watchOS(.v9),
        .tvOS(.v16)
    ],
	products: [
		.library(
			name: "YttriumCore",
			targets: ["YttriumCore"]
		),
	],
	dependencies: [],
	targets: [
		.binaryTarget(
			name: "RustXcframework",
			path: "RustXcframework.xcframework"
		),
		.target(
			name: "YttriumCore",
			dependencies: [
                "RustXcframework"
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
