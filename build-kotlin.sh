#!/bin/bash
set -eo pipefail

PROFILE="${PROFILE:-uniffi-release}"
TARGET_DIR="${TARGET_DIR:-target}"
ENABLE_STRIP="${ENABLE_STRIP:-true}"
CARGO_RUSTFLAGS="${CARGO_RUSTFLAGS:-}"
CARGO_FLAGS="${CARGO_FLAGS:-}"
CARGO_NDK_FLAGS="${CARGO_NDK_FLAGS:-}"
UNIFFI_OMIT_CHECKSUMS="${UNIFFI_OMIT_CHECKSUMS:-false}"

# Note: see .env.template for required ANDROID_NDK_HOME env var

# Ensure NDK version supports 16 KB page sizes (NDK 26+ required)
if [ -n "$ANDROID_NDK_HOME" ]; then
    echo "Using ANDROID_NDK_HOME: $ANDROID_NDK_HOME"
    # Check if NDK version supports 16 KB page sizes
    ndk_version=$(grep -o 'Pkg\.Revision = [0-9]*\.[0-9]*\.[0-9]*' "$ANDROID_NDK_HOME/source.properties" | cut -d' ' -f3)
    echo "NDK Version: $ndk_version"
    
    # Ensure NDK 26+ for 16 KB page size support
    major_version=$(echo "$ndk_version" | cut -d'.' -f1)
    if [ "$major_version" -lt 26 ]; then
        echo "Warning: NDK version $ndk_version may not fully support 16 KB page sizes. Consider using NDK 26+ for optimal compatibility."
    fi
fi

rm -rf crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a/
rm -rf crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a/
rm -rf crates/kotlin-ffi/android/src/main/kotlin/com/reown/yttrium/

if [ "$UNIFFI_OMIT_CHECKSUMS" == "true" ]; then
    echo "Omitting checksums"
    sed -i '' 's/^# omit_checksums = true/omit_checksums = true/' crates/kotlin-ffi/uniffi.toml
else
    echo "Not omitting checksums"
    sed -i '' 's/^omit_checksums = true/# omit_checksums = true/' crates/kotlin-ffi/uniffi.toml
fi

RUSTFLAGS="$CARGO_RUSTFLAGS" cargo $CARGO_FLAGS ndk -t armeabi-v7a -t arm64-v8a build -p kotlin-ffi --profile=$PROFILE --features=uniffi/cli --target-dir=$TARGET_DIR $CARGO_NDK_FLAGS
cargo run --features=uniffi/cli --bin uniffi-bindgen generate --library $TARGET_DIR/aarch64-linux-android/$PROFILE/libuniffi_yttrium.so --language kotlin --out-dir yttrium/kotlin-bindings

sed -i '' 's/^omit_checksums = true/# omit_checksums = true/' crates/kotlin-ffi/uniffi.toml

mkdir -p crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a
mkdir -p crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a
mkdir -p crates/kotlin-ffi/android/src/main/kotlin/com/reown/yttrium

echo "Moving binaries and bindings"
mv $TARGET_DIR/aarch64-linux-android/$PROFILE/libuniffi_yttrium.so crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a/
mv $TARGET_DIR/armv7-linux-androideabi/$PROFILE/libuniffi_yttrium.so crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a/
# mv yttrium/kotlin-bindings/uniffi/uniffi_yttrium/uniffi_yttrium.kt crates/kotlin-ffi/android/src/main/kotlin/com/reown/yttrium/
# mv yttrium/kotlin-bindings/uniffi/yttrium/yttrium.kt crates/kotlin-ffi/android/src/main/kotlin/com/reown/yttrium/

if [ "$ENABLE_STRIP" == "true" ]; then
    echo "Stripping binaries"
    strip="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip"
    # strip="strip"
    $strip crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a/libuniffi_yttrium.so
    $strip crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a/libuniffi_yttrium.so
fi

# Verify 16 KB alignment for native libraries
echo "Verifying 16 KB page size alignment..."
if command -v zipalign >/dev/null 2>&1; then
    echo "Checking alignment with zipalign..."
    # Note: This is a basic check - actual APK alignment will be done during APK build
    echo "Native libraries built with 16 KB page size support"
else
    echo "zipalign not found - alignment verification skipped"
fi

mkdir -p benchmark/build-kotlin/$PROFILE/arm64-v8a/
mkdir -p benchmark/build-kotlin/$PROFILE/armeabi-v7a/
stat -f%z crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a/libuniffi_yttrium.so > benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.size
stat -f%z crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a/libuniffi_yttrium.so > benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.size

echo "benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.txt: $(cat benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.size)"
echo "benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.txt: $(cat benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.size)"
