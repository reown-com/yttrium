#!/bin/bash
# run this script from inside /crates/yttrium_dart/

# cd rust
# cd ..
# cd yttrium

rustup target add armv7-linux-androideabi aarch64-linux-android
cargo ndk -t armeabi-v7a -t arm64-v8a build --profile=uniffi-release --features=frb -p yttrium

# cd .. #/yttrium_dart

mkdir -p android/src/main/jniLibs/arm64-v8a
mkdir -p android/src/main/jniLibs/armeabi-v7a
# mkdir -p android/src/main/jniLibs/x86
# mkdir -p android/src/main/jniLibs/x86_64

cd .. #/crates
cd .. #/yttrium

cp target/aarch64-linux-android/uniffi-release/libyttrium.so crates/yttrium_dart/android/src/main/jniLibs/arm64-v8a/libyttrium.so
cp target/armv7-linux-androideabi/uniffi-release/libyttrium.so crates/yttrium_dart/android/src/main/jniLibs/armeabi-v7a/libyttrium.so
# cp target/i686-linux-android/uniffi-release/libyttrium.so crates/yttrium_dart/android/src/main/jniLibs/x86/libyttrium.so
# cp target/x86_64-linux-android/uniffi-release/libyttrium.so crates/yttrium_dart/android/src/main/jniLibs/x86_64/libyttrium.so
