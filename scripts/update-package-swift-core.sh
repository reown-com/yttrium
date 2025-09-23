#!/bin/bash

set -e

# Script to update only YttriumCore configuration in Package.swift
# This ensures we don't overwrite YttriumUtils configuration

VERSION=$1
CHECKSUM=$2

if [ -z "$VERSION" ] || [ -z "$CHECKSUM" ]; then
    echo "Usage: $0 <version> <checksum>"
    echo "Example: $0 0.9.46 abc123..."
    exit 1
fi

echo "Updating YttriumCore configuration in Package.swift..."
echo "Version: $VERSION"
echo "Checksum: $CHECKSUM"

# Update only the YttriumCore URL line (line with libyttrium.xcframework.zip)
sed -i '' "s|url: \"https://github.com/reown-com/yttrium/releases/download/.*/libyttrium.xcframework.zip\"|url: \"https://github.com/reown-com/yttrium/releases/download/$VERSION/libyttrium.xcframework.zip\"|" Package.swift

# Update only the YttriumCore checksum line (the line immediately after the YttriumCore URL)
# Find the YttriumCore URL line number, then update the next line
CORE_URL_LINE=$(grep -n "libyttrium.xcframework.zip" Package.swift | head -1 | cut -d: -f1)
CORE_CHECKSUM_LINE=$((CORE_URL_LINE + 1))

# Update the checksum on the line after the YttriumCore URL
sed -i '' "${CORE_CHECKSUM_LINE}s/checksum: \".*\"/checksum: \"$CHECKSUM\"/" Package.swift

echo "✅ YttriumCore configuration updated successfully!"
echo "YttriumUtils configuration left unchanged." 