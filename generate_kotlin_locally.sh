#!/bin/bash
set -eo pipefail

rm -rf crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a/
rm -rf crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a/
rm -rf crates/kotlin-ffi/android/src/main/kotlin/com/reown/yttrium/

cargo ndk -t armv7-linux-androideabi -t aarch64-linux-android build --profile=uniffi-release-kotlin --features=uniffi/cli
cargo run --features=uniffi/cli --bin uniffi-bindgen generate --library target/aarch64-linux-android/uniffi-release-kotlin/libuniffi_yttrium.so --language kotlin --out-dir yttrium/kotlin-bindings

mkdir -p crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a
mkdir -p crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a
mkdir -p crates/kotlin-ffi/android/src/main/kotlin/com/reown/yttrium

echo "Moving binaries and bindings"
mv target/aarch64-linux-android/uniffi-release-kotlin/libuniffi_yttrium.so crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a/
mv target/armv7-linux-androideabi/uniffi-release-kotlin/libuniffi_yttrium.so crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a/
mv yttrium/kotlin-bindings/uniffi/uniffi_yttrium/uniffi_yttrium.kt crates/kotlin-ffi/android/src/main/kotlin/com/reown/yttrium/
mv yttrium/kotlin-bindings/uniffi/yttrium/yttrium.kt crates/kotlin-ffi/android/src/main/kotlin/com/reown/yttrium/

$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a/libuniffi_yttrium.so
$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a/libuniffi_yttrium.so

gradle clean assembleRelease publishToMavenLocal
