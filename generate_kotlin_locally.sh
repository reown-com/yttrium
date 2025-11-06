#!/bin/bash
set -eo pipefail

# Check if ANDROID_NDK_HOME is set
if [ -z "$ANDROID_NDK_HOME" ]; then
    echo "ERROR: ANDROID_NDK_HOME is not set!"
    echo "Please set it before running this script:"
    echo "  export ANDROID_NDK_HOME=\$HOME/Library/Android/sdk/ndk/<version>"
    echo ""
    echo "Available NDK versions:"
    ls -1 "$HOME/Library/Android/sdk/ndk/" 2>/dev/null || echo "  No NDK found at $HOME/Library/Android/sdk/ndk/"
    exit 1
fi

echo "Using ANDROID_NDK_HOME: $ANDROID_NDK_HOME"

ACCOUNT_FEATURES="android,erc6492_client,uniffi/cli"
UTILS_FEATURES="android,chain_abstraction_client,solana,stacks,sui,ton,eip155,uniffi/cli"
PROFILE="uniffi-release-kotlin"
OUTPUT_ROOT="build/kotlin-artifacts"
GEN_ROOT="crates/kotlin-ffi/android/build/generated"
TARGETS=("aarch64-linux-android" "armv7-linux-androideabi" "x86_64-linux-android")

abi_name() {
    case "$1" in
        aarch64-linux-android) echo "arm64-v8a" ;;
        armv7-linux-androideabi) echo "armeabi-v7a" ;;
        x86_64-linux-android) echo "x86_64" ;;
        *) echo "" ;;
    esac
}

cleanup() {
    echo "Cleaning build artifacts..."
    
    # Remove output artifacts
    rm -rf "$OUTPUT_ROOT"
    rm -rf yttrium/kotlin-bindings
    rm -rf yttrium/kotlin-utils-bindings
    
    # Remove generated sources and JNI libs (ensures fresh stripped binaries are used)
    rm -rf "$GEN_ROOT"
    
    # Remove entire Gradle build directory to prevent cached unstripped binaries
    rm -rf crates/kotlin-ffi/android/build

    # Purge any legacy sources/libs in src/main to avoid mixing with generated flavor inputs
    rm -rf crates/kotlin-ffi/android/src/main/jniLibs/arm64-v8a
    rm -rf crates/kotlin-ffi/android/src/main/jniLibs/armeabi-v7a
    rm -rf crates/kotlin-ffi/android/src/main/jniLibs/x86_64
    rm -rf crates/kotlin-ffi/android/src/main/kotlin/com/reown/yttrium
    
    echo "Cleanup complete"
}

copy_bindings() {
    local source_dir=$1
    local destination_dir=$2

    mkdir -p "$destination_dir"
    if [ -d "$source_dir" ]; then
        cp -R "$source_dir/." "$destination_dir/"
    fi
}

copy_library_variants() {
    local profile=$1
    local binary_name=$2
    local destination_root=$3

    for target in "${TARGETS[@]}"; do
        local abi
        abi="$(abi_name "$target")"
        if [ -z "$abi" ]; then
            echo "Unknown ABI for target $target"
            exit 1
        fi
        local src="target/${target}/${profile}/${binary_name}"
        if [ ! -f "$src" ]; then
            echo "Missing expected library at $src"
            exit 1
        fi
        
        # Strip the binary before copying
        if [ -n "$ANDROID_NDK_HOME" ]; then
            local strip_bin=""
            if [ -f "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip" ]; then
                strip_bin="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip"
            elif [ -f "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip" ]; then
                strip_bin="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip"
            else
                strip_bin=$(find "$ANDROID_NDK_HOME" -name "llvm-strip" -print -quit 2>/dev/null || true)
            fi
            
            if [ -n "$strip_bin" ]; then
                echo "Stripping $src with $strip_bin"
                "$strip_bin" "$src"
            fi
        fi
        
        mkdir -p "$destination_root/libs/${abi}"
        cp "$src" "$destination_root/libs/${abi}/${binary_name}"
    done
}

install_variant_sources() {
    local variant=$1
    local bindings_dir=$2

    local jni_base="$GEN_ROOT/${variant}/jniLibs"
    local kotlin_base="$GEN_ROOT/${variant}/kotlin/com/reown/yttrium"
    local wrapper_base="$GEN_ROOT/${variant}/kotlin/com/yttrium"
    local library_name="libuniffi_yttrium.so"
    local system_library="uniffi_yttrium"

    if [ "$variant" = "utils" ]; then
        library_name="libuniffi_yttrium_utils.so"
        system_library="uniffi_yttrium_utils"
    fi

    rm -rf "$jni_base" "$kotlin_base" "$wrapper_base"

    for target in "${TARGETS[@]}"; do
        local abi
        abi="$(abi_name "$target")"
        local src="target/${target}/${PROFILE}/libuniffi_yttrium.so"
        mkdir -p "$jni_base/${abi}"
        cp "$src" "$jni_base/${abi}/${library_name}"
    done

    mkdir -p "$kotlin_base"
    cp "${bindings_dir}/uniffi/uniffi_yttrium/uniffi_yttrium.kt" "$kotlin_base/"
    cp "${bindings_dir}/uniffi/yttrium/yttrium.kt" "$kotlin_base/"

    if [ "$variant" = "utils" ]; then
        # Change generated Kotlin package names and library name to avoid class duplication when both AARs are added
        
        # Process uniffi_yttrium.kt:
        # 1. Change package: uniffi.uniffi_yttrium -> uniffi.uniffi_yttrium_utils
        # 2. Change imports: uniffi.yttrium -> uniffi.yttrium_utils (but NOT uniffi.yttrium_utils)
        # 3. Change library name: "uniffi_yttrium" -> "uniffi_yttrium_utils"
        if command -v perl >/dev/null 2>&1; then
            perl -0pi -e 's/\bpackage\s+uniffi\.uniffi_yttrium\b/package uniffi.uniffi_yttrium_utils/g' "$kotlin_base/uniffi_yttrium.kt"
            perl -0pi -e 's/\buniffi\.yttrium(?!_utils)\b/uniffi.yttrium_utils/g' "$kotlin_base/uniffi_yttrium.kt"
            perl -0pi -e 's/return "uniffi_yttrium"/return "uniffi_yttrium_utils"/g' "$kotlin_base/uniffi_yttrium.kt"
        else
            sed -i '' 's/^package uniffi\.uniffi_yttrium$/package uniffi.uniffi_yttrium_utils/' "$kotlin_base/uniffi_yttrium.kt" || true
            sed -i '' 's/uniffi\.yttrium\([^_]\)/uniffi.yttrium_utils\1/g' "$kotlin_base/uniffi_yttrium.kt" || true
            sed -i '' 's/uniffi\.yttrium$/uniffi.yttrium_utils/g' "$kotlin_base/uniffi_yttrium.kt" || true
            sed -i '' 's/return "uniffi_yttrium"/return "uniffi_yttrium_utils"/g' "$kotlin_base/uniffi_yttrium.kt" || true
        fi

        # Process yttrium.kt:
        # 1. Change package: uniffi.yttrium -> uniffi.yttrium_utils
        # 2. Change imports: uniffi.uniffi_yttrium -> uniffi.uniffi_yttrium_utils
        # 3. Change library name: "uniffi_yttrium" -> "uniffi_yttrium_utils"
        if command -v perl >/dev/null 2>&1; then
            perl -0pi -e 's/\bpackage\s+uniffi\.yttrium\b/package uniffi.yttrium_utils/g' "$kotlin_base/yttrium.kt"
            perl -0pi -e 's/\buniffi\.uniffi_yttrium(?!_utils)\b/uniffi.uniffi_yttrium_utils/g' "$kotlin_base/yttrium.kt"
            perl -0pi -e 's/return "uniffi_yttrium"/return "uniffi_yttrium_utils"/g' "$kotlin_base/yttrium.kt"
        else
            sed -i '' 's/^package uniffi\.yttrium$/package uniffi.yttrium_utils/' "$kotlin_base/yttrium.kt" || true
            sed -i '' 's/uniffi\.uniffi_yttrium\([^_]\)/uniffi.uniffi_yttrium_utils\1/g' "$kotlin_base/yttrium.kt" || true
            sed -i '' 's/uniffi\.uniffi_yttrium$/uniffi.uniffi_yttrium_utils/g' "$kotlin_base/yttrium.kt" || true
            sed -i '' 's/return "uniffi_yttrium"/return "uniffi_yttrium_utils"/g' "$kotlin_base/yttrium.kt" || true
        fi
    fi
}

build_account_variant() {
    echo "Building yttrium (erc6492_client) variant..."
    cargo ndk -t armv7-linux-androideabi -t aarch64-linux-android -t x86_64-linux-android build \
        --profile="$PROFILE" \
        --no-default-features \
        --features="$ACCOUNT_FEATURES" \
        -p kotlin-ffi

    cargo run \
        --no-default-features \
        --features="$ACCOUNT_FEATURES" \
        -p kotlin-ffi \
        --bin uniffi-bindgen generate \
        --library "target/aarch64-linux-android/${PROFILE}/libuniffi_yttrium.so" \
        --language kotlin \
        --out-dir yttrium/kotlin-bindings

    mkdir -p "$OUTPUT_ROOT"
    copy_library_variants "$PROFILE" "libuniffi_yttrium.so" "$OUTPUT_ROOT"
    copy_bindings yttrium/kotlin-bindings "$OUTPUT_ROOT/kotlin-bindings"
    install_variant_sources "yttrium" "yttrium/kotlin-bindings"
}

build_utils_variant() {
    echo "Building utils variant (solana, stacks, sui, ton)..."
    cargo ndk -t armv7-linux-androideabi -t aarch64-linux-android -t x86_64-linux-android build \
        --profile="$PROFILE" \
        --no-default-features \
        --features="$UTILS_FEATURES" \
        -p kotlin-ffi

    cargo run \
        --no-default-features \
        --features="$UTILS_FEATURES" \
        -p kotlin-ffi \
        --bin uniffi-bindgen generate \
        --library "target/aarch64-linux-android/${PROFILE}/libuniffi_yttrium.so" \
        --language kotlin \
        --out-dir yttrium/kotlin-utils-bindings

    mkdir -p "$OUTPUT_ROOT"
    
    # Find llvm-strip once for all targets
    local strip_bin=""
    if [ -n "$ANDROID_NDK_HOME" ]; then
        if [ -f "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip" ]; then
            strip_bin="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip"
        elif [ -f "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip" ]; then
            strip_bin="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip"
        else
            strip_bin=$(find "$ANDROID_NDK_HOME" -name "llvm-strip" -print -quit 2>/dev/null || true)
        fi
    fi
    
    for target in "${TARGETS[@]}"; do
        local abi
        abi="$(abi_name "$target")"
        local src="target/${target}/${PROFILE}/libuniffi_yttrium.so"
        
        # Strip the binary before copying
        if [ -n "$strip_bin" ]; then
            echo "Stripping $src with $strip_bin"
            "$strip_bin" "$src"
        fi
        
        mkdir -p "$OUTPUT_ROOT/libs/${abi}"
        cp "$src" "$OUTPUT_ROOT/libs/${abi}/libuniffi_yttrium_utils.so"
    done

    copy_bindings yttrium/kotlin-utils-bindings "$OUTPUT_ROOT/kotlin-utils-bindings"
    install_variant_sources "utils" "yttrium/kotlin-utils-bindings"
}

strip_binaries() {
    if [ -z "$ANDROID_NDK_HOME" ]; then
        echo "Warning: ANDROID_NDK_HOME not set, skipping strip step"
        return
    fi

    echo "Using ANDROID_NDK_HOME: $ANDROID_NDK_HOME"

    local strip_bin=""
    if [ -f "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip" ]; then
        strip_bin="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip"
    elif [ -f "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip" ]; then
        strip_bin="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip"
    else
        echo "Searching for llvm-strip in $ANDROID_NDK_HOME..."
        strip_bin=$(find "$ANDROID_NDK_HOME" -name "llvm-strip" -print -quit 2>/dev/null || true)
    fi

    if [ -z "$strip_bin" ]; then
        echo "Warning: Could not find llvm-strip, skipping strip step"
        return
    fi

    echo "Found llvm-strip at: $strip_bin"

    local libs_to_strip=(
        "$GEN_ROOT/yttrium/jniLibs/arm64-v8a/libuniffi_yttrium.so"
        "$GEN_ROOT/yttrium/jniLibs/armeabi-v7a/libuniffi_yttrium.so"
        "$GEN_ROOT/utils/jniLibs/arm64-v8a/libuniffi_yttrium_utils.so"
        "$GEN_ROOT/utils/jniLibs/armeabi-v7a/libuniffi_yttrium_utils.so"
        "$OUTPUT_ROOT/libs/arm64-v8a/libuniffi_yttrium.so"
        "$OUTPUT_ROOT/libs/armeabi-v7a/libuniffi_yttrium.so"
        "$OUTPUT_ROOT/libs/arm64-v8a/libuniffi_yttrium_utils.so"
        "$OUTPUT_ROOT/libs/armeabi-v7a/libuniffi_yttrium_utils.so"
    )

    for lib in "${libs_to_strip[@]}"; do
        if [ -f "$lib" ]; then
            # Full strip to minimize binary size (matching 0.9.55 release)
            "$strip_bin" "$lib"
        fi
    done
}

cleanup
build_account_variant
build_utils_variant
strip_binaries

echo "Kotlin artifacts generated under $OUTPUT_ROOT"

./gradlew \
  assembleYttriumRelease \
  assembleUtilsRelease \
  publishYttriumReleasePublicationToMavenLocal \
  publishYttriumUtilsReleasePublicationToMavenLocal
