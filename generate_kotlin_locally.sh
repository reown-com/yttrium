#!/bin/bash
set -eo pipefail

rm -rf crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a/
rm -rf crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a/
rm -rf crates/kotlin-ffi/android/src/main/kotlin/com/reown/yttrium/

cargo ndk -t armv7-linux-androideabi -t aarch64-linux-android build --profile=profile1 --features=android,uniffi/cli -p kotlin-ffi
cargo run --features=android,uniffi/cli -p kotlin-ffi --bin uniffi-bindgen generate --library target/aarch64-linux-android/profile1/libuniffi_yttrium.so --language kotlin --out-dir yttrium/kotlin-bindings

mkdir -p crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a
mkdir -p crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a
mkdir -p crates/kotlin-ffi/android/src/main/kotlin/com/reown/yttrium

echo "Moving binaries and bindings"
mv target/aarch64-linux-android/profile1/libuniffi_yttrium.so crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a/
mv target/armv7-linux-androideabi/profile1/libuniffi_yttrium.so crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a/
mv yttrium/kotlin-bindings/uniffi/uniffi_yttrium/uniffi_yttrium.kt crates/kotlin-ffi/android/src/main/kotlin/com/reown/yttrium/
mv yttrium/kotlin-bindings/uniffi/yttrium/yttrium.kt crates/kotlin-ffi/android/src/main/kotlin/com/reown/yttrium/

if [ -n "$ANDROID_NDK_HOME" ]; then
    echo "Using ANDROID_NDK_HOME: $ANDROID_NDK_HOME"
    
    # Check NDK version for 16 KB page size support
    if [ -f "$ANDROID_NDK_HOME/source.properties" ]; then
        ndk_version=$(grep -o 'Pkg\.Revision = [0-9]*\.[0-9]*\.[0-9]*' "$ANDROID_NDK_HOME/source.properties" | cut -d' ' -f3)
        echo "NDK Version: $ndk_version"
        
        major_version=$(echo "$ndk_version" | cut -d'.' -f1)
        if [ "$major_version" -lt 26 ]; then
            echo "Warning: NDK version $ndk_version may not fully support 16 KB page sizes. Consider using NDK 26+ for optimal compatibility."
        fi
    fi
    
    if [ -f "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip" ]; then
        echo "Found llvm-strip at: $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip"
        $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a/libuniffi_yttrium.so
        $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a/libuniffi_yttrium.so

    elif [ -f "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip" ]; then
        echo "Found llvm-strip at: $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip"
        $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a/libuniffi_yttrium.so
        $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a/libuniffi_yttrium.so

    else
        echo "Searching for llvm-strip in $ANDROID_NDK_HOME..."
        find "$ANDROID_NDK_HOME" -name "llvm-strip" 2>/dev/null || echo "No llvm-strip found"
    fi
else
    echo "Warning: ANDROID_NDK_HOME not set, skipping strip step"
fi

echo "Kotlin bindings generated with 16 KB page size support for Android 15+ compatibility"
echo "Note: Built only for ARM64 (aarch64-linux-android) as it's the primary architecture for modern Android devices and supports 16 KB page sizes"

./gradlew clean assembleRelease publishToMavenLocal