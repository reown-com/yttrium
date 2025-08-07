#!/bin/bash

set -e

# Variables
RUST_XCFRAMEWORK_DIR="target/ios-utils/libyttrium-utils.xcframework"
RUST_XCFRAMEWORK_ZIP="libyttrium-utils.xcframework.zip"
OUTPUT_DIR="Output"

echo "Zipping Rust XCFramework..."
mkdir -p $OUTPUT_DIR
cd $RUST_XCFRAMEWORK_DIR
zip -r "../../../$OUTPUT_DIR/$RUST_XCFRAMEWORK_ZIP" .
cd ../../../

echo "Utils XCFramework zipped successfully at $OUTPUT_DIR/$RUST_XCFRAMEWORK_ZIP" 