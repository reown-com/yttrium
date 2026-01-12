#!/bin/bash
set -eo pipefail

# Script to benchmark Swift/iOS binary sizes across different Cargo profiles
# Tests various optimization settings and outputs CSV for comparison
#
# Usage: ./scripts/benchmark-swift-profiles.sh

OUTPUT_FILE="benchmark-swift-profile-sizes.csv"
PACKAGE_NAME="yttrium"
FEATURES="ios,pay"
TARGET="aarch64-apple-ios"

echo "=== Swift/iOS Profile Size Benchmark ==="
echo "Output: $OUTPUT_FILE"
echo ""

# CSV header
echo "profile,libyttrium.a (device),libyttrium.a (sim-x86),libyttrium.a (sim-arm)" > $OUTPUT_FILE

# Function to get file size (cross-platform)
get_file_size() {
    local file=$1
    if [ -f "$file" ]; then
        if [[ "$OSTYPE" == "darwin"* ]]; then
            stat -f%z "$file"
        else
            stat -c%s "$file"
        fi
    else
        echo "N/A"
    fi
}

# Function to build for all iOS targets with a given profile
build_ios_targets() {
    local profile=$1

    # Build for aarch64-apple-ios (Physical Devices)
    echo "  Building aarch64-apple-ios..."
    export CC_aarch64_apple_ios="$(xcrun --sdk iphoneos --find clang)"
    export AR_aarch64_apple_ios="$(xcrun --sdk iphoneos --find ar)"
    export CARGO_TARGET_AARCH64_APPLE_IOS_LINKER="$CC_aarch64_apple_ios"
    export IPHONEOS_DEPLOYMENT_TARGET="13.0"
    export CFLAGS_aarch64_apple_ios="-miphoneos-version-min=13.0"
    export RUSTFLAGS="-C linker=$CC_aarch64_apple_ios -C link-arg=-miphoneos-version-min=13.0"

    cargo build \
        --lib --profile=$profile \
        --no-default-features \
        --features=$FEATURES \
        --target aarch64-apple-ios \
        -p $PACKAGE_NAME 2>/dev/null

    unset CC_aarch64_apple_ios AR_aarch64_apple_ios CARGO_TARGET_AARCH64_APPLE_IOS_LINKER
    unset IPHONEOS_DEPLOYMENT_TARGET CFLAGS_aarch64_apple_ios RUSTFLAGS

    # Build for x86_64-apple-ios (Simulator on Intel Macs)
    echo "  Building x86_64-apple-ios..."
    export CC_x86_64_apple_ios="$(xcrun --sdk iphonesimulator --find clang)"
    export AR_x86_64_apple_ios="$(xcrun --sdk iphonesimulator --find ar)"
    export CARGO_TARGET_X86_64_APPLE_IOS_LINKER="$CC_x86_64_apple_ios"
    export IPHONEOS_DEPLOYMENT_TARGET="13.0"
    export CFLAGS_x86_64_apple_ios="-mios-simulator-version-min=13.0"
    export RUSTFLAGS="-C linker=$CC_x86_64_apple_ios -C link-arg=-mios-simulator-version-min=13.0"

    cargo build \
        --lib --profile=$profile \
        --no-default-features \
        --features=$FEATURES \
        --target x86_64-apple-ios \
        -p $PACKAGE_NAME 2>/dev/null

    unset CC_x86_64_apple_ios AR_x86_64_apple_ios CARGO_TARGET_X86_64_APPLE_IOS_LINKER
    unset IPHONEOS_DEPLOYMENT_TARGET CFLAGS_x86_64_apple_ios RUSTFLAGS

    # Build for aarch64-apple-ios-sim (Simulator on Apple Silicon Macs)
    echo "  Building aarch64-apple-ios-sim..."
    export CC_aarch64_apple_ios_sim="$(xcrun --sdk iphonesimulator --find clang)"
    export AR_aarch64_apple_ios_sim="$(xcrun --sdk iphonesimulator --find ar)"
    export CARGO_TARGET_AARCH64_APPLE_IOS_SIM_LINKER="$CC_aarch64_apple_ios_sim"
    export IPHONEOS_DEPLOYMENT_TARGET="13.0"
    export CFLAGS_aarch64_apple_ios_sim="-mios-simulator-version-min=13.0"
    export RUSTFLAGS="-C linker=$CC_aarch64_apple_ios_sim -C link-arg=-mios-simulator-version-min=13.0"

    cargo build \
        --lib --profile=$profile \
        --no-default-features \
        --features=$FEATURES \
        --target aarch64-apple-ios-sim \
        -p $PACKAGE_NAME 2>/dev/null

    unset CC_aarch64_apple_ios_sim AR_aarch64_apple_ios_sim CARGO_TARGET_AARCH64_APPLE_IOS_SIM_LINKER
    unset IPHONEOS_DEPLOYMENT_TARGET CFLAGS_aarch64_apple_ios_sim RUSTFLAGS
}

# Function to measure and record sizes for a profile
measure_profile() {
    local profile=$1
    local label=${2:-$profile}

    echo "Building profile: $label"
    build_ios_targets $profile

    local device_lib="target/aarch64-apple-ios/$profile/lib$PACKAGE_NAME.a"
    local sim_x86_lib="target/x86_64-apple-ios/$profile/lib$PACKAGE_NAME.a"
    local sim_arm_lib="target/aarch64-apple-ios-sim/$profile/lib$PACKAGE_NAME.a"

    local size1=$(get_file_size "$device_lib")
    local size2=$(get_file_size "$sim_x86_lib")
    local size3=$(get_file_size "$sim_arm_lib")

    echo "$label,$size1,$size2,$size3" >> $OUTPUT_FILE
    echo "  Device: $size1, Sim-x86: $size2, Sim-arm: $size3"
    echo ""
}

# Ensure iOS targets are available
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim 2>/dev/null || true

# Test standard profiles (matching Android benchmark)
echo "=== Testing Standard Profiles ==="
profiles="profile6 profile8 profile9 profile10"
for profile in $profiles; do
    measure_profile $profile
done

# Test the production Swift profile
echo "=== Testing Production Swift Profile ==="
measure_profile "uniffi-release-swift"

# Test nightly builds with build-std optimizations (requires nightly toolchain)
if rustup run nightly rustc --version >/dev/null 2>&1; then
    echo "=== Testing Nightly Profiles with build-std ==="

    nightly_profiles="profile8 profile9 profile10"
    for profile in $nightly_profiles; do
        echo "Building profile: $profile-nightly-stdopt"

        # Build with nightly and build-std optimizations
        echo "  Building aarch64-apple-ios with build-std..."
        export CC_aarch64_apple_ios="$(xcrun --sdk iphoneos --find clang)"
        export AR_aarch64_apple_ios="$(xcrun --sdk iphoneos --find ar)"
        export CARGO_TARGET_AARCH64_APPLE_IOS_LINKER="$CC_aarch64_apple_ios"
        export IPHONEOS_DEPLOYMENT_TARGET="13.0"
        export CFLAGS_aarch64_apple_ios="-miphoneos-version-min=13.0"
        export RUSTFLAGS="-C linker=$CC_aarch64_apple_ios -C link-arg=-miphoneos-version-min=13.0"

        cargo +nightly build \
            -Z build-std=std,panic_abort \
            -Z build-std-features="optimize_for_size" \
            --lib --profile=$profile \
            --no-default-features \
            --features=$FEATURES \
            --target aarch64-apple-ios \
            --target-dir="target/nightly-stdopt" \
            -p $PACKAGE_NAME 2>/dev/null || echo "  (nightly build failed, skipping)"

        unset CC_aarch64_apple_ios AR_aarch64_apple_ios CARGO_TARGET_AARCH64_APPLE_IOS_LINKER
        unset IPHONEOS_DEPLOYMENT_TARGET CFLAGS_aarch64_apple_ios RUSTFLAGS

        local device_lib="target/nightly-stdopt/aarch64-apple-ios/$profile/lib$PACKAGE_NAME.a"
        if [ -f "$device_lib" ]; then
            local size1=$(get_file_size "$device_lib")
            echo "$profile-nightly-stdopt,$size1,N/A,N/A" >> $OUTPUT_FILE
            echo "  Device: $size1"
        fi
        echo ""
    done

    # Test with extra nightly flags
    echo "=== Testing Nightly with Extra Flags ==="
    for profile in $nightly_profiles; do
        echo "Building profile: $profile-nightly-stdopt-extra"

        export CC_aarch64_apple_ios="$(xcrun --sdk iphoneos --find clang)"
        export AR_aarch64_apple_ios="$(xcrun --sdk iphoneos --find ar)"
        export CARGO_TARGET_AARCH64_APPLE_IOS_LINKER="$CC_aarch64_apple_ios"
        export IPHONEOS_DEPLOYMENT_TARGET="13.0"
        export CFLAGS_aarch64_apple_ios="-miphoneos-version-min=13.0"
        export RUSTFLAGS="-C linker=$CC_aarch64_apple_ios -C link-arg=-miphoneos-version-min=13.0 -Zlocation-detail=none -Zfmt-debug=none"

        cargo +nightly build \
            -Z build-std=std,panic_abort \
            -Z build-std-features="optimize_for_size" \
            --lib --profile=$profile \
            --no-default-features \
            --features=$FEATURES \
            --target aarch64-apple-ios \
            --target-dir="target/nightly-stdopt-extra" \
            -p $PACKAGE_NAME 2>/dev/null || echo "  (nightly build failed, skipping)"

        unset CC_aarch64_apple_ios AR_aarch64_apple_ios CARGO_TARGET_AARCH64_APPLE_IOS_LINKER
        unset IPHONEOS_DEPLOYMENT_TARGET CFLAGS_aarch64_apple_ios RUSTFLAGS

        local device_lib="target/nightly-stdopt-extra/aarch64-apple-ios/$profile/lib$PACKAGE_NAME.a"
        if [ -f "$device_lib" ]; then
            local size1=$(get_file_size "$device_lib")
            echo "$profile-nightly-stdopt-extra,$size1,N/A,N/A" >> $OUTPUT_FILE
            echo "  Device: $size1"
        fi
        echo ""
    done
else
    echo "Nightly toolchain not available, skipping nightly builds"
fi

echo "=== Benchmark Complete ==="
echo "Results saved to: $OUTPUT_FILE"
echo ""
cat $OUTPUT_FILE
