#!/bin/bash
set -eo pipefail

# Script to measure Swift/iOS static library sizes and XCFramework size
# Usage: ./scripts/measure-swift-size.sh [output-dir]

OUTPUT_DIR="${1:-target/ios}"
RESULTS_FILE="${OUTPUT_DIR}/swift-size-report.txt"

# Ensure output directory exists
mkdir -p "$OUTPUT_DIR"

echo "=== Swift/iOS Size Measurement Report ===" > "$RESULTS_FILE"
echo "Generated: $(date)" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

# Function to format bytes to human-readable format
format_size() {
    local bytes=$1
    if command -v numfmt >/dev/null 2>&1; then
        numfmt --to=iec-i --suffix=B "$bytes"
    else
        # Fallback for systems without numfmt (macOS)
        if [ "$bytes" -lt 1024 ]; then
            echo "${bytes}B"
        elif [ "$bytes" -lt 1048576 ]; then
            echo "$((bytes / 1024))KB"
        else
            echo "$((bytes / 1048576))MB"
        fi
    fi
}

# Function to get file size (cross-platform)
get_file_size() {
    local file=$1
    if [[ "$OSTYPE" == "darwin"* ]]; then
        stat -f%z "$file"
    else
        stat -c%s "$file"
    fi
}

total_size=0
arch_count=0

PROFILE="uniffi-release-swift"

echo "=== Static Library Sizes (.a files) ===" >> "$RESULTS_FILE"
echo "Profile: $PROFILE" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

# Check each architecture
# aarch64-apple-ios (Physical Devices)
device_lib="target/aarch64-apple-ios/$PROFILE/libyttrium.a"
if [ -f "$device_lib" ]; then
    size=$(get_file_size "$device_lib")
    formatted=$(format_size "$size")
    echo "  aarch64-apple-ios (device):     ${formatted} (${size} bytes)" >> "$RESULTS_FILE"
    total_size=$((total_size + size))
    arch_count=$((arch_count + 1))
else
    echo "  aarch64-apple-ios (device):     NOT FOUND" >> "$RESULTS_FILE"
fi

# x86_64-apple-ios (Simulator on Intel Macs)
sim_x86_lib="target/x86_64-apple-ios/$PROFILE/libyttrium.a"
if [ -f "$sim_x86_lib" ]; then
    size=$(get_file_size "$sim_x86_lib")
    formatted=$(format_size "$size")
    echo "  x86_64-apple-ios (sim Intel):   ${formatted} (${size} bytes)" >> "$RESULTS_FILE"
    total_size=$((total_size + size))
    arch_count=$((arch_count + 1))
else
    echo "  x86_64-apple-ios (sim Intel):   NOT FOUND" >> "$RESULTS_FILE"
fi

# aarch64-apple-ios-sim (Simulator on Apple Silicon Macs)
sim_arm_lib="target/aarch64-apple-ios-sim/$PROFILE/libyttrium.a"
if [ -f "$sim_arm_lib" ]; then
    size=$(get_file_size "$sim_arm_lib")
    formatted=$(format_size "$size")
    echo "  aarch64-apple-ios-sim (sim M1): ${formatted} (${size} bytes)" >> "$RESULTS_FILE"
    total_size=$((total_size + size))
    arch_count=$((arch_count + 1))
else
    echo "  aarch64-apple-ios-sim (sim M1): NOT FOUND" >> "$RESULTS_FILE"
fi

echo "" >> "$RESULTS_FILE"
echo "Total .a size (all archs): $(format_size $total_size) ($total_size bytes)" >> "$RESULTS_FILE"
echo "Number of architectures: $arch_count" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

# Check fat simulator library
fat_lib="target/ios-simulator-fat/$PROFILE/libyttrium.a"
if [ -f "$fat_lib" ]; then
    fat_size=$(get_file_size "$fat_lib")
    echo "=== Fat Simulator Library (x86_64 + arm64) ===" >> "$RESULTS_FILE"
    echo "  Fat lib: $(format_size $fat_size) ($fat_size bytes)" >> "$RESULTS_FILE"
    echo "" >> "$RESULTS_FILE"
fi

# Check XCFramework
xcframework_dir="target/ios/libyttrium.xcframework"
if [ -d "$xcframework_dir" ]; then
    # Calculate total XCFramework size
    if [[ "$OSTYPE" == "darwin"* ]]; then
        xcframework_size=$(du -sk "$xcframework_dir" | cut -f1)
        xcframework_size=$((xcframework_size * 1024))
    else
        xcframework_size=$(du -sb "$xcframework_dir" | cut -f1)
    fi
    echo "=== XCFramework Size ===" >> "$RESULTS_FILE"
    echo "  XCFramework: $(format_size $xcframework_size) ($xcframework_size bytes)" >> "$RESULTS_FILE"
    echo "" >> "$RESULTS_FILE"

    # List contents
    echo "  Contents:" >> "$RESULTS_FILE"
    for slice_dir in "$xcframework_dir"/*/; do
        if [ -d "$slice_dir" ]; then
            slice_name=$(basename "$slice_dir")
            lib_file="$slice_dir/libyttrium.a"
            if [ -f "$lib_file" ]; then
                slice_size=$(get_file_size "$lib_file")
                echo "    $slice_name: $(format_size $slice_size)" >> "$RESULTS_FILE"
            fi
        fi
    done
    echo "" >> "$RESULTS_FILE"
fi

# Display results
cat "$RESULTS_FILE"

echo ""
echo "Full report saved to: $RESULTS_FILE"
