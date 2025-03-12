#!/bin/bash
# run this script from inside /crates/yttrium_dart/

set -e

rm -Rf ios/yttrium_dart.xcframework

rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim
# rustup target add x86_64-apple-ios

cargo build -p yttrium --features=frb --target aarch64-apple-ios --profile=dart-release-v1
cargo build -p yttrium --features=frb --target aarch64-apple-ios-sim --profile=dart-release-v1
# cargo build -p yttrium --features=frb --target x86_64-apple-ios --profile=dart-release-v1

cd .. #/crates
cd .. #/yttrium

xcodebuild -create-xcframework \
    -library target/aarch64-apple-ios/dart-release-v1/libyttrium.a \
    -library target/aarch64-apple-ios-sim/dart-release-v1/libyttrium.a \
    -output crates/yttrium_dart/ios/yttrium_dart.xcframework


cd crates/yttrium_dart/example

# rm -Rf .dart_tool
# rm -Rf build
# rm -Rf ios/.symlinks
# rm -Rf ios/Pods
# rm -Rf ios/Runner.xcworkspace
# rm -Rf ios/Podfile.lock

flutter clean
flutter pub get

cd ios
pod deintegrate && pod cache clean --all && pod install