#!/usr/bin/env zsh

set -e
set -u

PACKAGE_NAME="uniffi_yttrium"
fat_simulator_lib_dir="target/ios-simulator-fat/release"
swift_package_dir="platforms/swift/Sources/Yttrium"

generate_ffi() {
  echo "Generating framework module mapping and FFI bindings..."
  cargo run --features uniffi/cli --bin uniffi-bindgen generate \
      --library target/aarch64-apple-ios/release/lib$1.dylib \
      --language swift \
      --out-dir target/uniffi-xcframework-staging

echo "creating module.modulemap"
cat target/uniffi-xcframework-staging/yttriumFFI.modulemap \
    target/uniffi-xcframework-staging/uniffi_yttriumFFI.modulemap \
    > target/uniffi-xcframework-staging/module.modulemap

  echo "Copying bindings to Swift package directory..."
  mkdir -p "$swift_package_dir"
  cp target/uniffi-xcframework-staging/*.swift "$swift_package_dir/"
  cp target/uniffi-xcframework-staging/*.h "$swift_package_dir/"
}

create_fat_simulator_lib() {
  echo "Creating a fat library for x86_64 and aarch64 simulators..."
  mkdir -p "$fat_simulator_lib_dir"
  lipo -create \
      target/x86_64-apple-ios/release/lib$1.a \
      target/aarch64-apple-ios-sim/release/lib$1.a \
      -output "$fat_simulator_lib_dir/lib$1.a"
}

build_xcframework() {
  echo "Generating XCFramework..."
  rm -rf target/ios
  mkdir -p target/ios
  xcodebuild -create-xcframework \
      -library target/aarch64-apple-ios/release/lib$1.a -headers target/uniffi-xcframework-staging \
      -library "$fat_simulator_lib_dir/lib$1.a" -headers target/uniffi-xcframework-staging \
      -output target/ios/lib$1.xcframework
}

rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios
rustup target add aarch64-apple-ios-sim

cargo build --release --target aarch64-apple-ios
cargo build --release --target x86_64-apple-ios
cargo build --release --target aarch64-apple-ios-sim

generate_ffi $PACKAGE_NAME
create_fat_simulator_lib $PACKAGE_NAME
build_xcframework $PACKAGE_NAME
