#!/bin/bash
set -eo pipefail

PROFILE="${PROFILE:-uniffi-release}"
TARGET_DIR="${TARGET_DIR:-target}"
ENABLE_STRIP="${ENABLE_STRIP:-true}"
CARGO_FLAGS="${CARGO_FLAGS:-}"
CARGO_NDK_FLAGS="${CARGO_NDK_FLAGS:-}"

# TODO?
# export ANDROID_NDK_HOME="/Users/chris13524/Library/Android/sdk/ndk/28.0.12674087/"

make build-xcframework
make CONFIG=debug build-swift-apple-platforms

mkdir -p benchmark/build-swift/$PROFILE/arm64-v8a/
mkdir -p benchmark/build-swift/$PROFILE/armeabi-v7a/
stat -f%z crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a/libuniffi_yttrium.so > benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.size
stat -f%z crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a/libuniffi_yttrium.so > benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.size

echo "benchmark/build-swift/$PROFILE/arm64-v8a/libuniffi_yttrium.so.txt: $(cat benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.size)"
echo "benchmark/build-swift/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.txt: $(cat benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.size)"
