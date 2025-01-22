#!/bin/bash
set -eo pipefail

output="benchmark.csv"
echo "config,libuniffi_yttrium.so.arm64-v8a,libuniffi_yttrium.so.armeabi-v7a" > $output

export ENABLE_STRIP="false"

export PROFILE="uniffi-release"
# ./build-kotlin.sh
# echo "kotlin/$PROFILE/nostrip,$(cat benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.size),$(cat benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.size)" >> benchmark.csv

export PROFILE="uniffi-release-v2"
# ./build-kotlin.sh
# echo "kotlin/$PROFILE/nostrip,$(cat benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.size),$(cat benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.size)" >> benchmark.csv

export ENABLE_STRIP="true"

export PROFILE="uniffi-release"
# ./build-kotlin.sh
# echo "kotlin/$PROFILE,$(cat benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.size),$(cat benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.size)" >> benchmark.csv

export PROFILE="uniffi-release-v2"
# ./build-kotlin.sh
# echo "kotlin/$PROFILE,$(cat benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.size),$(cat benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.size)" >> benchmark.csv

export PROFILE="kotlin-release-next"
# ./build-kotlin.sh
# echo "kotlin/$PROFILE,$(cat benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.size),$(cat benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.size)" >> benchmark.csv

export CARGO_FLAGS="+nightly"
# ./build-kotlin.sh
# echo "kotlin/$PROFILE/nightly,$(cat benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.size),$(cat benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.size)" >> benchmark.csv

export CARGO_NDK_FLAGS="-Z build-std=std,panic_abort -Z build-std-features=optimize_for_size"

# TARGET_DIR="target/kotlin-build/$PROFILE/stdo" ./build-kotlin.sh
# echo "kotlin/$PROFILE/stdo,$(cat benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.size),$(cat benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.size)" >> benchmark.csv

export CARGO_RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none"

# TARGET_DIR="target/kotlin-build/$PROFILE/stdo-dld-nfd" ./build-kotlin.sh
# echo "kotlin/$PROFILE/stdo-dld-nfd,$(cat benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.size),$(cat benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.size)" >> benchmark.csv

export CARGO_NDK_FLAGS="-Z build-std=std,panic_abort -Z build-std-features=optimize_for_size,panic_immediate_abort"

TARGET_DIR="target/kotlin-build/$PROFILE/stdo-dld-nfd-pia" ./build-kotlin.sh
echo "kotlin/$PROFILE/stdo-nld-nfd-pia,$(cat benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.size),$(cat benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.size)" >> benchmark.csv

export UNIFFI_OMIT_CHECKSUMS="true"

TARGET_DIR="target/kotlin-build/$PROFILE/stdo-dld-nfd-pia-nuc" ./build-kotlin.sh
echo "kotlin/$PROFILE/stdo-nld-nfd-pia-nuc,$(cat benchmark/build-kotlin/$PROFILE/arm64-v8a/libuniffi_yttrium.so.size),$(cat benchmark/build-kotlin/$PROFILE/armeabi-v7a/libuniffi_yttrium.so.size)" >> benchmark.csv


echo "Building Swift"
# TODO Swift


echo "Building WASM"

# ./build-wasm.sh
# WASM_FLAGS="--dev" ./build-wasm.sh
# WASM_FLAGS="--release" ./build-wasm.sh
# TODO more WASM?
