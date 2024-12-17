#!/bin/bash
# run this script from inside /crates/yttrium_dart/

rm -r lib/generated/*
flutter_rust_bridge_codegen generate --config-file flutter_rust_bridge.yaml
