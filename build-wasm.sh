#!/bin/bash
set -eo pipefail

WASM_FLAGS="${WASM_FLAGS:-}"

cd crates/yttrium
wasm-pack build --target web --features=wasm $WASM_FLAGS
mkdir -p ../../benchmark/build-wasm/web/
stat -f%z pkg/yttrium_bg.wasm > ../../benchmark/build-wasm/web/yttrium_bg.wasm.size

wasm-pack build --target nodejs --features=wasm $WASM_FLAGS
mkdir -p ../../benchmark/build-wasm/nodejs/
stat -f%z pkg/yttrium_bg.wasm > ../../benchmark/build-wasm/nodejs/yttrium_bg.wasm.size
