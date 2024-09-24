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
