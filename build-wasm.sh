#!/bin/bash
set -eo pipefail

WASM_FLAGS="${WASM_FLAGS:-}"

cd crates/yttrium
RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" rustup run nightly wasm-pack build --target web --features=wasm $WASM_FLAGS -Z build-std=std,panic_abort -Z build-std-features=optimize_for_size,panic_immediate_abort
mkdir -p ../../benchmark/build-wasm/web/
stat -f%z pkg/yttrium_bg.wasm > ../../benchmark/build-wasm/web/yttrium_bg.wasm.size

# wasm-pack build --target nodejs --features=wasm $WASM_FLAGS
# mkdir -p ../../benchmark/build-wasm/nodejs/
# stat -f%z pkg/yttrium_bg.wasm > ../../benchmark/build-wasm/nodejs/yttrium_bg.wasm.size
