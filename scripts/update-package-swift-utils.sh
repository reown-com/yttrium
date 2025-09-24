#!/bin/bash

set -e

# Script to update only YttriumUtils configuration in Package.swift
# This ensures we don't overwrite YttriumCore configuration

VERSION=$1
CHECKSUM=$2

if [ -z "$VERSION" ] || [ -z "$CHECKSUM" ]; then
    echo "Usage: $0 <version> <checksum>"
    echo "Example: $0 0.0.2 xyz789..."
    exit 1
fi

echo "Updating YttriumUtils configuration in Package.swift..."
echo "Version: $VERSION"
echo "Checksum: $CHECKSUM"

# Update only the YttriumUtils URL line (line with libyttrium-utils.xcframework.zip)
sed -i '' "s|url: \"https://github.com/reown-com/yttrium/releases/download/.*/libyttrium-utils.xcframework.zip\"|url: \"https://github.com/reown-com/yttrium/releases/download/$VERSION/libyttrium-utils.xcframework.zip\"|" Package.swift

# Update only the YttriumUtils checksum line (the line immediately after the YttriumUtils URL)
# Find the YttriumUtils URL line number, then update the next line
UTILS_URL_LINE=$(grep -n "libyttrium-utils.xcframework.zip" Package.swift | head -1 | cut -d: -f1)
UTILS_CHECKSUM_LINE=$((UTILS_URL_LINE + 1))

# Update the checksum on the line after the YttriumUtils URL
sed -i '' "${UTILS_CHECKSUM_LINE}s/checksum: \".*\"/checksum: \"$CHECKSUM\"/" Package.swift

echo "✅ YttriumUtils configuration updated successfully!"
echo "YttriumCore configuration left unchanged." 