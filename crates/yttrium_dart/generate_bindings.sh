#!/bin/bash

# run this script from inside /crates/yttrium_dart/

# flutter pub get -v

# flutter_rust_bridge_codegen generate --config-file flutter_rust_bridge.yaml

# rustup target add aarch64-apple-ios x86_64-apple-ios

cargo clean
cargo build --manifest-path rust/Cargo.toml --target aarch64-apple-ios --release
cargo build --manifest-path rust/Cargo.toml --target x86_64-apple-ios --release

cd ..
cd ..

mkdir -p target/universal/release

lipo -create target/aarch64-apple-ios/release/libyttrium_dart.a target/x86_64-apple-ios/release/libyttrium_dart.a -output target/universal/release/libyttrium_dart_universal.a

lipo -info target/universal/release/libyttrium_dart_universal.a

cp target/universal/release/libyttrium_dart_universal.a crates/yttrium_dart/ios/Frameworks/libyttrium_dart_universal.a

cd crates/yttrium_dart/example

flutter pub get -v

cd ios

pod install