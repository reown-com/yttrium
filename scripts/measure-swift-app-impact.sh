#!/bin/bash
set -eo pipefail

# Measures actual app size impact of linking Yttrium by building real archives and IPAs.
#
# Usage: ./scripts/measure-swift-app-impact.sh [output-dir]
#
# Prerequisites:
#   - macOS with Xcode installed
#   - XCFramework must be built first: ./scripts/build-xcframework.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_APP_DIR="$PROJECT_ROOT/.github/test-app"
OUTPUT_DIR="${1:-$PROJECT_ROOT/target/ios}"
BUILD_DIR="$OUTPUT_DIR/app-size-test"
XCFRAMEWORK_PATH="$PROJECT_ROOT/target/ios/libyttrium.xcframework"

mkdir -p "$OUTPUT_DIR" "$BUILD_DIR"

format_size() {
    local bytes=$1
    if [ "$bytes" -lt 1048576 ]; then
        printf "%.1fKB" "$(echo "scale=1; $bytes / 1024" | bc)"
    else
        printf "%.2fMB" "$(echo "scale=2; $bytes / 1048576" | bc)"
    fi
}

get_file_size() { stat -f%z "$1"; }

# Build baseline app (without Yttrium)
build_baseline() {
    local build_path="$BUILD_DIR/baseline"
    local archive_path="$build_path/App.xcarchive"
    local ipa_path="$build_path/App.ipa"

    rm -rf "$build_path"
    mkdir -p "$build_path"

    echo "  Archiving baseline..." >&2
    xcodebuild archive \
        -project "$TEST_APP_DIR/YttriumSizeTest.xcodeproj" \
        -scheme YttriumSizeTest \
        -configuration Release \
        -destination 'generic/platform=iOS' \
        -archivePath "$archive_path" \
        CODE_SIGN_IDENTITY=- \
        CODE_SIGNING_REQUIRED=NO \
        CODE_SIGNING_ALLOWED=NO \
        ENABLE_BITCODE=NO \
        SKIP_INSTALL=NO \
        2>&1 | grep -E "(error:|BUILD)" >&2 || true

    if [ ! -d "$archive_path" ]; then
        echo "ERROR: Baseline archive failed" >&2
        return 1
    fi

    echo "  Exporting IPA..." >&2
    xcodebuild -exportArchive \
        -archivePath "$archive_path" \
        -exportPath "$build_path" \
        -exportOptionsPlist "$TEST_APP_DIR/export-options.plist" \
        2>&1 | grep -E "(error:|EXPORT)" >&2 || true

    local exported_ipa=$(find "$build_path" -name "*.ipa" -type f 2>/dev/null | head -1)
    if [ -n "$exported_ipa" ]; then
        mv "$exported_ipa" "$ipa_path"
    fi

    echo "$archive_path|$ipa_path"
}

# Create an Xcode project with Yttrium SPM dependency
create_yttrium_project() {
    local project_path=$1
    local project_dir="$project_path/YttriumSizeTest.xcodeproj"

    mkdir -p "$project_dir"

    # Write the pbxproj file with SPM package reference
    cat > "$project_dir/project.pbxproj" << 'PBXPROJ_EOF'
// !$*UTF8*$!
{
	archiveVersion = 1;
	classes = {
	};
	objectVersion = 56;
	objects = {

/* Begin PBXBuildFile section */
		B1000001233456780001 /* YttriumSizeTestApp.swift in Sources */ = {isa = PBXBuildFile; fileRef = B1000001233456780011 /* YttriumSizeTestApp.swift */; };
		B1000001233456780002 /* ContentView.swift in Sources */ = {isa = PBXBuildFile; fileRef = B1000001233456780012 /* ContentView.swift */; };
		B1000001233456780099 /* Yttrium in Frameworks */ = {isa = PBXBuildFile; productRef = B1000001233456780098 /* Yttrium */; };
/* End PBXBuildFile section */

/* Begin PBXFileReference section */
		B1000001233456780010 /* YttriumSizeTest.app */ = {isa = PBXFileReference; explicitFileType = wrapper.application; includeInIndex = 0; path = YttriumSizeTest.app; sourceTree = BUILT_PRODUCTS_DIR; };
		B1000001233456780011 /* YttriumSizeTestApp.swift */ = {isa = PBXFileReference; lastKnownFileType = sourcecode.swift; path = YttriumSizeTestApp.swift; sourceTree = "<group>"; };
		B1000001233456780012 /* ContentView.swift */ = {isa = PBXFileReference; lastKnownFileType = sourcecode.swift; path = ContentView.swift; sourceTree = "<group>"; };
		B1000001233456780013 /* Info.plist */ = {isa = PBXFileReference; lastKnownFileType = text.plist.xml; path = Info.plist; sourceTree = "<group>"; };
/* End PBXFileReference section */

/* Begin PBXFrameworksBuildPhase section */
		B100000123345678000D /* Frameworks */ = {
			isa = PBXFrameworksBuildPhase;
			buildActionMask = 2147483647;
			files = (
				B1000001233456780099 /* Yttrium in Frameworks */,
			);
			runOnlyForDeploymentPostprocessing = 0;
		};
/* End PBXFrameworksBuildPhase section */

/* Begin PBXGroup section */
		B1000001233456780003 = {
			isa = PBXGroup;
			children = (
				B1000001233456780004 /* YttriumSizeTest */,
				B100000123345678000F /* Products */,
			);
			sourceTree = "<group>";
		};
		B1000001233456780004 /* YttriumSizeTest */ = {
			isa = PBXGroup;
			children = (
				B1000001233456780011 /* YttriumSizeTestApp.swift */,
				B1000001233456780012 /* ContentView.swift */,
				B1000001233456780013 /* Info.plist */,
			);
			path = YttriumSizeTest;
			sourceTree = "<group>";
		};
		B100000123345678000F /* Products */ = {
			isa = PBXGroup;
			children = (
				B1000001233456780010 /* YttriumSizeTest.app */,
			);
			name = Products;
			sourceTree = "<group>";
		};
/* End PBXGroup section */

/* Begin PBXNativeTarget section */
		B100000123345678000E /* YttriumSizeTest */ = {
			isa = PBXNativeTarget;
			buildConfigurationList = B1000001233456780019 /* Build configuration list for PBXNativeTarget "YttriumSizeTest" */;
			buildPhases = (
				B100000123345678000C /* Sources */,
				B100000123345678000D /* Frameworks */,
				B1000001233456780014 /* Resources */,
			);
			buildRules = (
			);
			dependencies = (
			);
			name = YttriumSizeTest;
			packageProductDependencies = (
				B1000001233456780098 /* Yttrium */,
			);
			productName = YttriumSizeTest;
			productReference = B1000001233456780010 /* YttriumSizeTest.app */;
			productType = "com.apple.product-type.application";
		};
/* End PBXNativeTarget section */

/* Begin PBXProject section */
		B1000001233456780005 /* Project object */ = {
			isa = PBXProject;
			attributes = {
				BuildIndependentTargetsInParallel = 1;
				LastSwiftUpdateCheck = 1500;
				LastUpgradeCheck = 1500;
				TargetAttributes = {
					B100000123345678000E = {
						CreatedOnToolsVersion = 15.0;
					};
				};
			};
			buildConfigurationList = B1000001233456780006 /* Build configuration list for PBXProject "YttriumSizeTest" */;
			compatibilityVersion = "Xcode 14.0";
			developmentRegion = en;
			hasScannedForEncodings = 0;
			knownRegions = (
				en,
				Base,
			);
			mainGroup = B1000001233456780003;
			packageReferences = (
				B1000001233456780097 /* XCLocalSwiftPackageReference "YTTRIUM_PATH_PLACEHOLDER" */,
			);
			productRefGroup = B100000123345678000F /* Products */;
			projectDirPath = "";
			projectRoot = "";
			targets = (
				B100000123345678000E /* YttriumSizeTest */,
			);
		};
/* End PBXProject section */

/* Begin PBXResourcesBuildPhase section */
		B1000001233456780014 /* Resources */ = {
			isa = PBXResourcesBuildPhase;
			buildActionMask = 2147483647;
			files = (
			);
			runOnlyForDeploymentPostprocessing = 0;
		};
/* End PBXResourcesBuildPhase section */

/* Begin PBXSourcesBuildPhase section */
		B100000123345678000C /* Sources */ = {
			isa = PBXSourcesBuildPhase;
			buildActionMask = 2147483647;
			files = (
				B1000001233456780001 /* YttriumSizeTestApp.swift in Sources */,
				B1000001233456780002 /* ContentView.swift in Sources */,
			);
			runOnlyForDeploymentPostprocessing = 0;
		};
/* End PBXSourcesBuildPhase section */

/* Begin XCBuildConfiguration section */
		B1000001233456780015 /* Debug */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				ALWAYS_SEARCH_USER_PATHS = NO;
				ASSETCATALOG_COMPILER_GENERATE_SWIFT_ASSET_SYMBOL_EXTENSIONS = YES;
				CLANG_ANALYZER_NONNULL = YES;
				CLANG_ANALYZER_NUMBER_OBJECT_CONVERSION = YES_AGGRESSIVE;
				CLANG_CXX_LANGUAGE_STANDARD = "gnu++20";
				CLANG_ENABLE_MODULES = YES;
				CLANG_ENABLE_OBJC_ARC = YES;
				CLANG_ENABLE_OBJC_WEAK = YES;
				CLANG_WARN_BLOCK_CAPTURE_AUTORELEASING = YES;
				CLANG_WARN_BOOL_CONVERSION = YES;
				CLANG_WARN_COMMA = YES;
				CLANG_WARN_CONSTANT_CONVERSION = YES;
				CLANG_WARN_DEPRECATED_OBJC_IMPLEMENTATIONS = YES;
				CLANG_WARN_DIRECT_OBJC_ISA_USAGE = YES_ERROR;
				CLANG_WARN_DOCUMENTATION_COMMENTS = YES;
				CLANG_WARN_EMPTY_BODY = YES;
				CLANG_WARN_ENUM_CONVERSION = YES;
				CLANG_WARN_INFINITE_RECURSION = YES;
				CLANG_WARN_INT_CONVERSION = YES;
				CLANG_WARN_NON_LITERAL_NULL_CONVERSION = YES;
				CLANG_WARN_OBJC_IMPLICIT_RETAIN_SELF = YES;
				CLANG_WARN_OBJC_LITERAL_CONVERSION = YES;
				CLANG_WARN_OBJC_ROOT_CLASS = YES_ERROR;
				CLANG_WARN_QUOTED_INCLUDE_IN_FRAMEWORK_HEADER = YES;
				CLANG_WARN_RANGE_LOOP_ANALYSIS = YES;
				CLANG_WARN_STRICT_PROTOTYPES = YES;
				CLANG_WARN_SUSPICIOUS_MOVE = YES;
				CLANG_WARN_UNGUARDED_AVAILABILITY = YES_AGGRESSIVE;
				CLANG_WARN_UNREACHABLE_CODE = YES;
				CLANG_WARN__DUPLICATE_METHOD_MATCH = YES;
				COPY_PHASE_STRIP = NO;
				DEBUG_INFORMATION_FORMAT = dwarf;
				ENABLE_STRICT_OBJC_MSGSEND = YES;
				ENABLE_TESTABILITY = YES;
				ENABLE_USER_SCRIPT_SANDBOXING = YES;
				GCC_C_LANGUAGE_STANDARD = gnu17;
				GCC_DYNAMIC_NO_PIC = NO;
				GCC_NO_COMMON_BLOCKS = YES;
				GCC_OPTIMIZATION_LEVEL = 0;
				GCC_PREPROCESSOR_DEFINITIONS = (
					"DEBUG=1",
					"$(inherited)",
				);
				GCC_WARN_64_TO_32_BIT_CONVERSION = YES;
				GCC_WARN_ABOUT_RETURN_TYPE = YES_ERROR;
				GCC_WARN_UNDECLARED_SELECTOR = YES;
				GCC_WARN_UNINITIALIZED_AUTOS = YES_AGGRESSIVE;
				GCC_WARN_UNUSED_FUNCTION = YES;
				GCC_WARN_UNUSED_VARIABLE = YES;
				IPHONEOS_DEPLOYMENT_TARGET = 14.0;
				LOCALIZATION_PREFERS_STRING_CATALOGS = YES;
				MTL_ENABLE_DEBUG_INFO = INCLUDE_SOURCE;
				MTL_FAST_MATH = YES;
				ONLY_ACTIVE_ARCH = YES;
				SDKROOT = iphoneos;
				SWIFT_ACTIVE_COMPILATION_CONDITIONS = "DEBUG $(inherited)";
				SWIFT_OPTIMIZATION_LEVEL = "-Onone";
			};
			name = Debug;
		};
		B1000001233456780016 /* Release */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				ALWAYS_SEARCH_USER_PATHS = NO;
				ASSETCATALOG_COMPILER_GENERATE_SWIFT_ASSET_SYMBOL_EXTENSIONS = YES;
				CLANG_ANALYZER_NONNULL = YES;
				CLANG_ANALYZER_NUMBER_OBJECT_CONVERSION = YES_AGGRESSIVE;
				CLANG_CXX_LANGUAGE_STANDARD = "gnu++20";
				CLANG_ENABLE_MODULES = YES;
				CLANG_ENABLE_OBJC_ARC = YES;
				CLANG_ENABLE_OBJC_WEAK = YES;
				CLANG_WARN_BLOCK_CAPTURE_AUTORELEASING = YES;
				CLANG_WARN_BOOL_CONVERSION = YES;
				CLANG_WARN_COMMA = YES;
				CLANG_WARN_CONSTANT_CONVERSION = YES;
				CLANG_WARN_DEPRECATED_OBJC_IMPLEMENTATIONS = YES;
				CLANG_WARN_DIRECT_OBJC_ISA_USAGE = YES_ERROR;
				CLANG_WARN_DOCUMENTATION_COMMENTS = YES;
				CLANG_WARN_EMPTY_BODY = YES;
				CLANG_WARN_ENUM_CONVERSION = YES;
				CLANG_WARN_INFINITE_RECURSION = YES;
				CLANG_WARN_INT_CONVERSION = YES;
				CLANG_WARN_NON_LITERAL_NULL_CONVERSION = YES;
				CLANG_WARN_OBJC_IMPLICIT_RETAIN_SELF = YES;
				CLANG_WARN_OBJC_LITERAL_CONVERSION = YES;
				CLANG_WARN_OBJC_ROOT_CLASS = YES_ERROR;
				CLANG_WARN_QUOTED_INCLUDE_IN_FRAMEWORK_HEADER = YES;
				CLANG_WARN_RANGE_LOOP_ANALYSIS = YES;
				CLANG_WARN_STRICT_PROTOTYPES = YES;
				CLANG_WARN_SUSPICIOUS_MOVE = YES;
				CLANG_WARN_UNGUARDED_AVAILABILITY = YES_AGGRESSIVE;
				CLANG_WARN_UNREACHABLE_CODE = YES;
				CLANG_WARN__DUPLICATE_METHOD_MATCH = YES;
				COPY_PHASE_STRIP = NO;
				DEBUG_INFORMATION_FORMAT = "dwarf-with-dsym";
				ENABLE_NS_ASSERTIONS = NO;
				ENABLE_STRICT_OBJC_MSGSEND = YES;
				ENABLE_USER_SCRIPT_SANDBOXING = YES;
				GCC_C_LANGUAGE_STANDARD = gnu17;
				GCC_NO_COMMON_BLOCKS = YES;
				GCC_WARN_64_TO_32_BIT_CONVERSION = YES;
				GCC_WARN_ABOUT_RETURN_TYPE = YES_ERROR;
				GCC_WARN_UNDECLARED_SELECTOR = YES;
				GCC_WARN_UNINITIALIZED_AUTOS = YES_AGGRESSIVE;
				GCC_WARN_UNUSED_FUNCTION = YES;
				GCC_WARN_UNUSED_VARIABLE = YES;
				IPHONEOS_DEPLOYMENT_TARGET = 14.0;
				LOCALIZATION_PREFERS_STRING_CATALOGS = YES;
				MTL_ENABLE_DEBUG_INFO = NO;
				MTL_FAST_MATH = YES;
				SDKROOT = iphoneos;
				SWIFT_COMPILATION_MODE = wholemodule;
				VALIDATE_PRODUCT = YES;
			};
			name = Release;
		};
		B1000001233456780017 /* Debug */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				ASSETCATALOG_COMPILER_APPICON_NAME = AppIcon;
				ASSETCATALOG_COMPILER_GLOBAL_ACCENT_COLOR_NAME = AccentColor;
				CODE_SIGN_IDENTITY = "-";
				CODE_SIGN_STYLE = Manual;
				CURRENT_PROJECT_VERSION = 1;
				DEVELOPMENT_TEAM = "";
				ENABLE_PREVIEWS = YES;
				GENERATE_INFOPLIST_FILE = YES;
				INFOPLIST_FILE = YttriumSizeTest/Info.plist;
				INFOPLIST_KEY_UIApplicationSceneManifest_Generation = YES;
				INFOPLIST_KEY_UIApplicationSupportsIndirectInputEvents = YES;
				INFOPLIST_KEY_UILaunchScreen_Generation = YES;
				INFOPLIST_KEY_UISupportedInterfaceOrientations = UIInterfaceOrientationPortrait;
				LD_RUNPATH_SEARCH_PATHS = (
					"$(inherited)",
					"@executable_path/Frameworks",
				);
				MARKETING_VERSION = 1.0;
				PRODUCT_BUNDLE_IDENTIFIER = com.reown.YttriumSizeTest;
				PRODUCT_NAME = "$(TARGET_NAME)";
				PROVISIONING_PROFILE_SPECIFIER = "";
				SWIFT_EMIT_LOC_STRINGS = YES;
				SWIFT_VERSION = 5.0;
				TARGETED_DEVICE_FAMILY = "1,2";
			};
			name = Debug;
		};
		B1000001233456780018 /* Release */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				ASSETCATALOG_COMPILER_APPICON_NAME = AppIcon;
				ASSETCATALOG_COMPILER_GLOBAL_ACCENT_COLOR_NAME = AccentColor;
				CODE_SIGN_IDENTITY = "-";
				CODE_SIGN_STYLE = Manual;
				CURRENT_PROJECT_VERSION = 1;
				DEVELOPMENT_TEAM = "";
				ENABLE_PREVIEWS = YES;
				GENERATE_INFOPLIST_FILE = YES;
				INFOPLIST_FILE = YttriumSizeTest/Info.plist;
				INFOPLIST_KEY_UIApplicationSceneManifest_Generation = YES;
				INFOPLIST_KEY_UIApplicationSupportsIndirectInputEvents = YES;
				INFOPLIST_KEY_UILaunchScreen_Generation = YES;
				INFOPLIST_KEY_UISupportedInterfaceOrientations = UIInterfaceOrientationPortrait;
				LD_RUNPATH_SEARCH_PATHS = (
					"$(inherited)",
					"@executable_path/Frameworks",
				);
				MARKETING_VERSION = 1.0;
				PRODUCT_BUNDLE_IDENTIFIER = com.reown.YttriumSizeTest;
				PRODUCT_NAME = "$(TARGET_NAME)";
				PROVISIONING_PROFILE_SPECIFIER = "";
				SWIFT_EMIT_LOC_STRINGS = YES;
				SWIFT_VERSION = 5.0;
				TARGETED_DEVICE_FAMILY = "1,2";
			};
			name = Release;
		};
/* End XCBuildConfiguration section */

/* Begin XCConfigurationList section */
		B1000001233456780006 /* Build configuration list for PBXProject "YttriumSizeTest" */ = {
			isa = XCConfigurationList;
			buildConfigurations = (
				B1000001233456780015 /* Debug */,
				B1000001233456780016 /* Release */,
			);
			defaultConfigurationIsVisible = 0;
			defaultConfigurationName = Release;
		};
		B1000001233456780019 /* Build configuration list for PBXNativeTarget "YttriumSizeTest" */ = {
			isa = XCConfigurationList;
			buildConfigurations = (
				B1000001233456780017 /* Debug */,
				B1000001233456780018 /* Release */,
			);
			defaultConfigurationIsVisible = 0;
			defaultConfigurationName = Release;
		};
/* End XCConfigurationList section */

/* Begin XCLocalSwiftPackageReference section */
		B1000001233456780097 /* XCLocalSwiftPackageReference "YTTRIUM_PATH_PLACEHOLDER" */ = {
			isa = XCLocalSwiftPackageReference;
			relativePath = "YTTRIUM_PATH_PLACEHOLDER";
		};
/* End XCLocalSwiftPackageReference section */

/* Begin XCSwiftPackageProductDependency section */
		B1000001233456780098 /* Yttrium */ = {
			isa = XCSwiftPackageProductDependency;
			package = B1000001233456780097 /* XCLocalSwiftPackageReference "YTTRIUM_PATH_PLACEHOLDER" */;
			productName = Yttrium;
		};
/* End XCSwiftPackageProductDependency section */
	};
	rootObject = B1000001233456780005 /* Project object */;
}
PBXPROJ_EOF

    # Replace placeholder with actual path
    sed -i '' "s|YTTRIUM_PATH_PLACEHOLDER|$PROJECT_ROOT|g" "$project_dir/project.pbxproj"
}

# Build app with Yttrium linked
build_with_yttrium() {
    local build_path="$BUILD_DIR/with-yttrium"
    local project_path="$build_path/YttriumSizeTest"
    local archive_path="$build_path/App.xcarchive"
    local ipa_path="$build_path/App.ipa"

    rm -rf "$build_path"
    mkdir -p "$project_path"

    # Copy test app sources
    cp -r "$TEST_APP_DIR/YttriumSizeTest" "$project_path/"
    cp "$TEST_APP_DIR/export-options.plist" "$project_path/"

    # Create ContentView that actually uses Yttrium (forces linker to include it)
    cat > "$project_path/YttriumSizeTest/ContentView.swift" << 'SWIFT_EOF'
import SwiftUI
import Yttrium

struct ContentView: View {
    var body: some View {
        VStack {
            Text("Yttrium Size Test")
                .font(.title)
            Text("Yttrium linked")
                .foregroundColor(.green)
        }
        .padding()
        .onAppear {
            // Force linker to include Yttrium by referencing a type
            let _ = SdkConfig(
                baseUrl: "https://example.com",
                projectId: "test",
                apiKey: "test",
                sdkName: "test",
                sdkVersion: "1.0",
                sdkPlatform: "iOS",
                bundleId: "com.test"
            )
        }
    }
}
SWIFT_EOF

    # Create Xcode project with SPM dependency
    echo "  Creating Xcode project with Yttrium dependency..." >&2
    create_yttrium_project "$project_path"

    echo "  Resolving Swift packages..." >&2
    xcodebuild -resolvePackageDependencies \
        -project "$project_path/YttriumSizeTest.xcodeproj" \
        -clonedSourcePackagesDirPath "$build_path/SourcePackages" \
        2>&1 | grep -E "(error:|Resolved)" >&2 || true

    echo "  Archiving with Yttrium..." >&2
    xcodebuild archive \
        -project "$project_path/YttriumSizeTest.xcodeproj" \
        -scheme YttriumSizeTest \
        -configuration Release \
        -destination 'generic/platform=iOS' \
        -archivePath "$archive_path" \
        -clonedSourcePackagesDirPath "$build_path/SourcePackages" \
        CODE_SIGN_IDENTITY=- \
        CODE_SIGNING_REQUIRED=NO \
        CODE_SIGNING_ALLOWED=NO \
        ENABLE_BITCODE=NO \
        SKIP_INSTALL=NO \
        2>&1 | grep -E "(error:|warning:|BUILD)" >&2 || true

    if [ ! -d "$archive_path" ]; then
        echo "ERROR: Yttrium archive failed" >&2
        return 1
    fi

    echo "  Exporting IPA..." >&2
    xcodebuild -exportArchive \
        -archivePath "$archive_path" \
        -exportPath "$build_path" \
        -exportOptionsPlist "$project_path/export-options.plist" \
        2>&1 | grep -E "(error:|EXPORT)" >&2 || true

    local exported_ipa=$(find "$build_path" -name "*.ipa" -type f 2>/dev/null | head -1)
    if [ -n "$exported_ipa" ]; then
        mv "$exported_ipa" "$ipa_path"
    fi

    echo "$archive_path|$ipa_path"
}

# Measure sizes from archive/IPA
measure_app() {
    local archive_path=$1
    local ipa_path=$2
    local binary_size=0
    local ipa_size=0

    local app_path="$archive_path/Products/Applications/YttriumSizeTest.app"
    local binary_path="$app_path/YttriumSizeTest"

    if [ -f "$binary_path" ]; then
        binary_size=$(get_file_size "$binary_path")
    fi

    if [ -f "$ipa_path" ]; then
        ipa_size=$(get_file_size "$ipa_path")
    fi

    echo "$binary_size|$ipa_size"
}

echo "=== iOS App Size Impact Measurement ==="
echo ""

# Check prerequisites
if [ ! -d "$XCFRAMEWORK_PATH" ]; then
    echo "ERROR: XCFramework not found at $XCFRAMEWORK_PATH"
    echo "Run ./scripts/build-xcframework.sh first."
    exit 1
fi

echo "Found XCFramework: $XCFRAMEWORK_PATH"
echo ""

# Build baseline
echo "=== Building Baseline (without Yttrium) ==="
baseline_result=$(build_baseline)
baseline_archive=$(echo "$baseline_result" | cut -d'|' -f1)
baseline_ipa=$(echo "$baseline_result" | cut -d'|' -f2)
baseline_sizes=$(measure_app "$baseline_archive" "$baseline_ipa")
baseline_binary=$(echo "$baseline_sizes" | cut -d'|' -f1)
baseline_ipa_size=$(echo "$baseline_sizes" | cut -d'|' -f2)

echo ""

# Build with Yttrium
echo "=== Building With Yttrium ==="
yttrium_result=$(build_with_yttrium)
yttrium_archive=$(echo "$yttrium_result" | cut -d'|' -f1)
yttrium_ipa=$(echo "$yttrium_result" | cut -d'|' -f2)
yttrium_sizes=$(measure_app "$yttrium_archive" "$yttrium_ipa")
yttrium_binary=$(echo "$yttrium_sizes" | cut -d'|' -f1)
yttrium_ipa_size=$(echo "$yttrium_sizes" | cut -d'|' -f2)

echo ""

# Calculate deltas
binary_delta=$((yttrium_binary - baseline_binary))
ipa_delta=$((yttrium_ipa_size - baseline_ipa_size))

# Output results
echo "=== Results ==="
echo ""
echo "Baseline:"
echo "  Binary: $(format_size $baseline_binary)"
if [ "$baseline_ipa_size" -gt 0 ]; then
    echo "  IPA:    $(format_size $baseline_ipa_size)"
else
    echo "  IPA:    (export failed - code signing required)"
fi
echo ""
echo "With Yttrium:"
echo "  Binary: $(format_size $yttrium_binary)"
if [ "$yttrium_ipa_size" -gt 0 ]; then
    echo "  IPA:    $(format_size $yttrium_ipa_size) (approx App Store download size)"
else
    echo "  IPA:    (export failed - code signing required)"
fi
echo ""
echo "Delta (Yttrium impact):"
echo "  Binary: +$(format_size $binary_delta)"
if [ "$baseline_ipa_size" -gt 0 ] && [ "$yttrium_ipa_size" -gt 0 ]; then
    echo "  IPA:    +$(format_size $ipa_delta)"
fi
echo ""
echo "Build artifacts: $BUILD_DIR"
