#!/bin/bash

# dart create -t package . --force

# cargo add flutter_rust_bridge@=1.82.6
# cargo add flutter_rust_bridge_codegen@=1.82.6

RUST_BACKTRACE=full flutter_rust_bridge_codegen generate --config-file flutter_rust_bridge.yaml