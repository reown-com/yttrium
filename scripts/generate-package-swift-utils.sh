#!/bin/bash

set -e

# Variables
RUST_XCFRAMEWORK_DIR="target/ios-utils/libyttrium-utils.xcframework"
RUST_XCFRAMEWORK_ZIP_SPM="libyttrium-utils.xcframework.zip"
RUST_XCFRAMEWORK_ZIP_POD="libyttrium-utils-pod.zip"
OUTPUT_DIR="Output"

echo "Preparing Utils release artifacts (SPM zip and Pod zip)..."
mkdir -p $OUTPUT_DIR

# 1) Create SPM artifact: pure XCFramework zip (keep the xcframework directory at top-level)
rm -f "$OUTPUT_DIR/$RUST_XCFRAMEWORK_ZIP_SPM"
(
  cd "$(dirname "$RUST_XCFRAMEWORK_DIR")"
  zip -r "../../$OUTPUT_DIR/$RUST_XCFRAMEWORK_ZIP_SPM" "$(basename "$RUST_XCFRAMEWORK_DIR")"
)
echo "SPM XCFramework zip created at $OUTPUT_DIR/$RUST_XCFRAMEWORK_ZIP_SPM"

# 2) Create CocoaPods artifact: XCFramework + Sources
TMPDIR=$(mktemp -d)
cp -R "$RUST_XCFRAMEWORK_DIR" "$TMPDIR/"
mkdir -p "$TMPDIR/Sources/YttriumUtils"
cp platforms/swift/Sources/YttriumUtils/*.swift "$TMPDIR/Sources/YttriumUtils/"
(
  cd "$TMPDIR"
  zip -r "$OLDPWD/$OUTPUT_DIR/$RUST_XCFRAMEWORK_ZIP_POD" .
)
rm -rf "$TMPDIR"

echo "Pod zip created at $OUTPUT_DIR/$RUST_XCFRAMEWORK_ZIP_POD (contains libyttrium-utils.xcframework and Sources/YttriumUtils)"