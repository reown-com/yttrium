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

echo "✅ $0: building for targest: x86_64-apple-ios $TARGET."

rustup target add x86_64-apple-ios $TARGET

cargo build -p yttrium --features=frb --target x86_64-apple-ios --profile=uniffi-release
cargo build -p yttrium --features=frb --target $TARGET --profile=uniffi-release

cd .. #/crates
cd .. #/yttrium

pwd

mkdir -p target/universal/ios/uniffi-release

lipo -create target/$TARGET/uniffi-release/libyttrium.dylib target/x86_64-apple-ios/uniffi-release/libyttrium.dylib -output target/universal/ios/uniffi-release/libyttrium_universal.dylib

lipo -info target/universal/ios/uniffi-release/libyttrium_universal.dylib

cp target/universal/ios/uniffi-release/libyttrium_universal.dylib crates/yttrium_dart/ios/libyttrium_universal.dylib

# otool -L crates/yttrium_dart/ios/libyttrium_universal.dylib

install_name_tool -id @rpath/libyttrium_universal.dylib crates/yttrium_dart/ios/libyttrium_universal.dylib

codesign --force --sign - crates/yttrium_dart/ios/libyttrium_universal.dylib

otool -L crates/yttrium_dart/ios/libyttrium_universal.dylib

pwd

cd crates/yttrium_dart/example
flutter clean
flutter pub get

cd ios
pod deintegrate && rm Podfile.lock && pod cache clean -all && pod install