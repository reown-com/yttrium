#!/bin/bash
set -eo pipefail

WASM_FLAGS="${WASM_FLAGS:-}"

wasm-pack build --target web $WASM_FLAGS
wasm-pack build --target nodejs $WASM_FLAGS

# mkdir -p benchmark/build-wasm/web/
# mkdir -p benchmark/build-wasm/web/
# stat -f%z crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a/libuniffi_yttrium.so > benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.size
# stat -f%z crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a/libuniffi_yttrium.so > benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.size

# echo "benchmark/build-wasm/$PROFILE/arm64-v8a/libuniffi_yttrium.so.txt: $(cat benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.size)"
# echo "benchmark/build-wasm/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.txt: $(cat benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.size)"
