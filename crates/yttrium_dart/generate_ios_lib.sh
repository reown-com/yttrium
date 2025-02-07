#!/bin/bash
# run this script from inside /crates/yttrium_dart/

DEFAULT_TARGET="aarch64-apple-ios"

if [ $# -eq 0 ];
then
  echo "✅ $0: No arguments passed, building for target: $DEFAULT_TARGET"
  TARGET=$DEFAULT_TARGET
  # exit 1
elif [ $# -gt 1 ];
then
  echo "❌ $0: Too many arguments, only one accepted. Arguments passed: $@"
  exit 1
else
  if [ $1 != "-sim" ];
  then
    echo "❌ $0: Wrong argument $1 passed. Only valid argument is '-sim'"
    exit 1
  else
    TARGET="aarch64-apple-ios$1"
    echo "✅ $0: Building for targe: $TARGET"
  fi
fi

cd rust

echo "✅ $0: building for targest: x86_64-apple-ios $TARGET."

rustup target add x86_64-apple-ios $TARGET

cargo build --manifest-path Cargo.toml --target x86_64-apple-ios --profile=uniffi-release
cargo build --manifest-path Cargo.toml --target $TARGET --profile=uniffi-release

cd .. #/yttrium_dart
cd .. #/crates
cd .. #/yttrium

mkdir -p target/universal/ios/uniffi-release

lipo -create target/$TARGET/uniffi-release/libyttrium_dart.dylib target/x86_64-apple-ios/uniffi-release/libyttrium_dart.dylib -output target/universal/ios/uniffi-release/libyttrium_dart_universal.dylib

lipo -info target/universal/ios/uniffi-release/libyttrium_dart_universal.dylib

cp target/universal/ios/uniffi-release/libyttrium_dart_universal.dylib crates/yttrium_dart/ios/libyttrium_dart_universal.dylib

# otool -L crates/yttrium_dart/ios/libyttrium_dart_universal.dylib

install_name_tool -id @rpath/libyttrium_dart_universal.dylib crates/yttrium_dart/ios/libyttrium_dart_universal.dylib

codesign --force --sign - crates/yttrium_dart/ios/libyttrium_dart_universal.dylib

# otool -L crates/yttrium_dart/ios/libyttrium_dart_universal.dylib

cd crates/yttrium_dart/example
flutter clean
flutter pub get

cd ios
pod deintegrate && rm Podfile.lock && pod cache clean -all && pod install