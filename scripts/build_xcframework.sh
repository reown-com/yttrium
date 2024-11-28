#!/usr/bin/env zsh

set -e
set -u

release=false

for arg in "$@"
do
    case $arg in
        --release)
            release=true
            shift # Remove --release from processing
            ;;
        *)
            shift # Ignore other argument from processing
            ;;
    esac
done

PACKAGE_NAME="uniffi_yttrium"
fat_simulator_lib_dir="target/ios-simulator-fat/release"
swift_package_dir="platforms/swift/Sources/Yttrium"

generate_ffi() {
  echo "Generating framework module mapping and FFI bindings..."
  cargo run --features uniffi/cli --bin uniffi-bindgen generate \
      --library target/aarch64-apple-ios/debug/lib$1.dylib \
      --language swift \
      --out-dir target/uniffi-xcframework-staging

  # Handle modulemap
#  if [ -f "target/uniffi-xcframework-staging/$1FFI.modulemap" ]; then
#      mv "target/uniffi-xcframework-staging/$1FFI.modulemap" "target/uniffi-xcframework-staging/module.modulemap"
#  fi

  echo "Copying bindings to Swift package directory..."
  mkdir -p "$swift_package_dir"
  cp target/uniffi-xcframework-staging/*.swift "$swift_package_dir/"
  cp target/uniffi-xcframework-staging/*.h "$swift_package_dir/"
  cp target/uniffi-xcframework-staging/*.modulemap "$swift_package_dir/" || echo "No modulemap to copy."
}

create_fat_simulator_lib() {
  echo "Creating a fat library for x86_64 and aarch64 simulators..."
  mkdir -p "$fat_simulator_lib_dir"
  lipo -create \
      target/x86_64-apple-ios/debug/lib$1.a \
      target/aarch64-apple-ios-sim/debug/lib$1.a \
      -output "$fat_simulator_lib_dir/lib$1.a"
}

build_xcframework() {
  echo "Generating XCFramework..."
  rm -rf target/ios
  mkdir -p target/ios
  xcodebuild -create-xcframework \
      -library target/aarch64-apple-ios/debug/lib$1.a -headers target/uniffi-xcframework-staging \
      -library "$fat_simulator_lib_dir/lib$1.a" -headers target/uniffi-xcframework-staging \
      -output target/ios/lib$1.xcframework

  if $release; then
      echo "Building xcframework archive..."
      ditto -c -k --sequesterRsrc --keepParent target/ios/lib$1.xcframework target/ios/lib$1.xcframework.zip
      checksum=$(swift package compute-checksum target/ios/lib$1.xcframework.zip)
      version=$(cargo metadata --format-version 1 | jq -r --arg pkg_name "$1" '.packages[] | select(.name==$pkg_name) .version')
      sed -i "" -E "s/(let releaseTag = \")[^\"]+(\")/\1$version\2/g" ../Package.swift
      sed -i "" -E "s/(let releaseChecksum = \")[^\"]+(\")/\1$checksum\2/g" ../Package.swift
  fi
}

# Build Rust libraries
#cargo build -p $PACKAGE_NAME --lib --release --target x86_64-apple-ios
#cargo build -p $PACKAGE_NAME --lib --release --target aarch64-apple-ios-sim
#cargo build -p $PACKAGE_NAME --lib --release --target aarch64-apple-ios

cargo build --target aarch64-apple-ios
cargo build --target x86_64-apple-ios
cargo build --target aarch64-apple-ios-sim

generate_ffi $PACKAGE_NAME
create_fat_simulator_lib $PACKAGE_NAME
build_xcframework $PACKAGE_NAME
