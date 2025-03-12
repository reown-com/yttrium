#!/bin/bash
# run this script from inside /crates/yttrium_dart/

rm -r lib/generated/*
# rm rust/src/frb_generated.rs
rm ../yttrium/src/frb_generated.rs

flutter clean
flutter pub get
# cargo clean
flutter_rust_bridge_codegen generate --config-file flutter_rust_bridge.yaml
# dart pub run build_runner build
