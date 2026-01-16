#!/bin/bash
set -eo pipefail

# Script to measure wcpay .so file sizes and AAR size
# Usage: ./scripts/measure-wcpay-size.sh [output-dir]

OUTPUT_DIR="${1:-build/kotlin-artifacts}"
RESULTS_FILE="${OUTPUT_DIR}/wcpay-size-report.txt"

echo "=== WCPay Size Measurement Report ===" > "$RESULTS_FILE"
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

total_size=0
abi_count=0

echo "=== Native Library Sizes (.so files) ===" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

for abi in arm64-v8a armeabi-v7a x86_64; do
    so_file="${OUTPUT_DIR}/libs/${abi}/libuniffi_yttrium_wcpay.so"
    if [ -f "$so_file" ]; then
        if [[ "$OSTYPE" == "darwin"* ]]; then
            size=$(stat -f%z "$so_file")
        else
            size=$(stat -c%s "$so_file")
        fi
        formatted=$(format_size "$size")
        echo "  ${abi}: ${formatted} (${size} bytes)" >> "$RESULTS_FILE"
        total_size=$((total_size + size))
        abi_count=$((abi_count + 1))
    else
        echo "  ${abi}: NOT FOUND" >> "$RESULTS_FILE"
    fi
done

echo "" >> "$RESULTS_FILE"
echo "Total .so size: $(format_size $total_size) ($total_size bytes)" >> "$RESULTS_FILE"
echo "Number of ABIs: $abi_count" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

# Check for AAR file
aar_file="${OUTPUT_DIR}/wcpay-release.aar"
if [ -f "$aar_file" ]; then
    if [[ "$OSTYPE" == "darwin"* ]]; then
        aar_size=$(stat -f%z "$aar_file")
    else
        aar_size=$(stat -c%s "$aar_file")
    fi
    echo "=== AAR Size ===" >> "$RESULTS_FILE"
    echo "  AAR: $(format_size $aar_size) ($aar_size bytes)" >> "$RESULTS_FILE"
    echo "" >> "$RESULTS_FILE"
fi

# Check Gradle build output
gradle_aar="crates/kotlin-ffi/android/build/outputs/aar/wcpay-release.aar"
if [ -f "$gradle_aar" ]; then
    if [[ "$OSTYPE" == "darwin"* ]]; then
        gradle_size=$(stat -f%z "$gradle_aar")
    else
        gradle_size=$(stat -c%s "$gradle_aar")
    fi
    echo "=== Gradle Build AAR Size ===" >> "$RESULTS_FILE"
    echo "  AAR: $(format_size $gradle_size) ($gradle_size bytes)" >> "$RESULTS_FILE"
    echo "" >> "$RESULTS_FILE"
fi

# Display results
cat "$RESULTS_FILE"

echo ""
echo "Full report saved to: $RESULTS_FILE"
