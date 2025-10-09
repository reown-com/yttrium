# WASM Build error

If you are using macOS arm64

https://github.com/briansmith/ring/issues/1824#issuecomment-2059955073

```text
warning: ring@0.17.14: error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-unknown"'
warning: ring@0.17.14: 1 error generated.
error: failed to run custom build command for `ring v0.17.14`

Caused by:
  process didn't exit successfully: `/Users/chris13524/git/github.com/reown-com/yttrium/target/debug/build/ring-a4c44c198c697dbc/build-script-build` (exit status: 1)
  --- stdout
  cargo:rerun-if-env-changed=CARGO_MANIFEST_DIR
  cargo:rerun-if-env-changed=CARGO_PKG_NAME
  cargo:rerun-if-env-changed=CARGO_PKG_VERSION_MAJOR
  cargo:rerun-if-env-changed=CARGO_PKG_VERSION_MINOR
  cargo:rerun-if-env-changed=CARGO_PKG_VERSION_PATCH
  cargo:rerun-if-env-changed=CARGO_PKG_VERSION_PRE
  cargo:rerun-if-env-changed=CARGO_MANIFEST_LINKS
  cargo:rerun-if-env-changed=RING_PREGENERATE_ASM
  cargo:rerun-if-env-changed=OUT_DIR
  cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH
  cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS
  cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ENV
  cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ENDIAN
  OPT_LEVEL = Some(0)
  OUT_DIR = Some(/Users/chris13524/git/github.com/reown-com/yttrium/target/wasm32-unknown-unknown/debug/build/ring-d536f3047373da8a/out)
  TARGET = Some(wasm32-unknown-unknown)
  HOST = Some(aarch64-apple-darwin)
  cargo:rerun-if-env-changed=CC_wasm32-unknown-unknown
  CC_wasm32-unknown-unknown = None
  cargo:rerun-if-env-changed=CC_wasm32_unknown_unknown
  CC_wasm32_unknown_unknown = None
  cargo:rerun-if-env-changed=TARGET_CC
  TARGET_CC = None
  cargo:rerun-if-env-changed=CC
  CC = None
  cargo:rerun-if-env-changed=CC_ENABLE_DEBUG_OUTPUT
  RUSTC_WRAPPER = None
  cargo:rerun-if-env-changed=CRATE_CC_NO_DEFAULTS
  CRATE_CC_NO_DEFAULTS = None
  DEBUG = Some(true)
  cargo:rerun-if-env-changed=CFLAGS
  CFLAGS = None
  cargo:rerun-if-env-changed=TARGET_CFLAGS
  TARGET_CFLAGS = None
  cargo:rerun-if-env-changed=CFLAGS_wasm32_unknown_unknown
  CFLAGS_wasm32_unknown_unknown = None
  cargo:rerun-if-env-changed=CFLAGS_wasm32-unknown-unknown
  CFLAGS_wasm32-unknown-unknown = None
  CARGO_ENCODED_RUSTFLAGS = Some(--cfggetrandom_backend="wasm_js")
  cargo:warning=error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-unknown"'
  cargo:warning=1 error generated.

  --- stderr


  error occurred in cc-rs: command did not execute successfully (status code exit status: 1): LC_ALL="C" "clang" "-O0" "-ffunction-sections" "-fdata-sections" "-fno-exceptions" "-g" "-fno-omit-frame-pointer" "--target=wasm32-unknown-unknown" "-I" "/Users/chris13524/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/include" "-I" "/Users/chris13524/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/pregenerated" "-Wall" "-Wextra" "-fvisibility=hidden" "-std=c1x" "-Wall" "-Wbad-function-cast" "-Wcast-align" "-Wcast-qual" "-Wconversion" "-Wmissing-field-initializers" "-Wmissing-include-dirs" "-Wnested-externs" "-Wredundant-decls" "-Wshadow" "-Wsign-compare" "-Wsign-conversion" "-Wstrict-prototypes" "-Wundef" "-Wuninitialized" "-g3" "-nostdlibinc" "-DNDEBUG" "-DRING_CORE_NOSTDLIBINC=1" "-o" "/Users/chris13524/git/github.com/reown-com/yttrium/target/wasm32-unknown-unknown/debug/build/ring-d536f3047373da8a/out/25ac62e5b3c53843-curve25519.o" "-c" "/Users/chris13524/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/crypto/curve25519/curve25519.c"
```
