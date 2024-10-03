#!/bin/bash

set -e

# Variables
: "${VERSION:?Error: VERSION environment variable is not set.}"
PACKAGE_VERSION="$VERSION"
RUST_CHECKSUM=$(cat rust_checksum.txt)
RUST_XCFRAMEWORK_ZIP="RustXcframework.xcframework.zip"
REPO_URL="https://github.com/reown-com/yttrium"

PACKAGE_FILE="Package.swift"

# Prepare the new URL and checksum
NEW_URL="$REPO_URL/releases/download/$PACKAGE_VERSION/$RUST_XCFRAMEWORK_ZIP"
NEW_CHECKSUM="$RUST_CHECKSUM"

# Use perl to update the URL and checksum in the remote binaryTarget
perl -i -pe '
    $in_target = 0;
    if (/let rustXcframeworkTarget: Target = useLocalRustXcframework \?/) {
        $in_target = 1;
    }
    if ($in_target && /\.binaryTarget\(/) {
        $in_binary_target = 1;
    }
    if ($in_binary_target && /url:\s*".*?"/) {
        s|url:\s*".*?"|url: "'"\"$NEW_URL\""'"|;
    }
    if ($in_binary_target && /checksum:\s*".*?"/) {
        s|checksum:\s*".*?"|checksum: "'"\"$NEW_CHECKSUM\""'"|;
        $in_binary_target = 0; # End after checksum
    }
' "$PACKAGE_FILE"

echo "Package.swift updated with new URL and checksum for RustXcframework."