#!/bin/bash
# run this script from inside /crates/yttrium_dart/

cd rust

rustup target add armv7-linux-androideabi aarch64-linux-android x86_64-linux-android i686-linux-android
cargo ndk -t armeabi-v7a -t arm64-v8a -t x86_64 -t x86 build --profile=uniffi-release

cd .. #/yttrium_dart

mkdir -p android/src/main/jniLibs/arm64-v8a
mkdir -p android/src/main/jniLibs/armeabi-v7a
# mkdir -p android/src/main/jniLibs/x86
# mkdir -p android/src/main/jniLibs/x86_64

cd .. #/crates
cd .. #/yttrium

cp target/aarch64-linux-android/uniffi-release/libyttrium_dart.so crates/yttrium_dart/android/src/main/jniLibs/arm64-v8a/libyttrium_dart.so
cp target/armv7-linux-androideabi/uniffi-release/libyttrium_dart.so crates/yttrium_dart/android/src/main/jniLibs/armeabi-v7a/libyttrium_dart.so
# cp target/i686-linux-android/uniffi-release/libyttrium_dart.so crates/yttrium_dart/android/src/main/jniLibs/x86/libyttrium_dart.so
# cp target/x86_64-linux-android/uniffi-release/libyttrium_dart.so crates/yttrium_dart/android/src/main/jniLibs/x86_64/libyttrium_dart.so
