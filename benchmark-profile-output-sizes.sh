#!/bin/bash
set -eo pipefail

# Pre-ran output can be found here and is updated periodically:
# https://docs.google.com/spreadsheets/d/1Ko7TmIbNrgB2PWN8cPDM4GL6Iy2d8c-VrD0FnQ9SicI/edit?usp=sharing

output="benchmark-profile-output-sizes.csv"
echo "profile,libuniffi_yttrium.a,libuniffi_yttrium.dylib,libuniffi_yttrium.so.aarch64,libuniffi_yttrium.so.armv7" > $output

# echo "debug"
# cargo build
# cargo ndk -t armeabi-v7a -t arm64-v8a build --features=uniffi/cli
# file1="target/debug/libuniffi_yttrium.a"
# file2="target/debug/libuniffi_yttrium.dylib"
# file3="target/aarch64-linux-android/debug/libuniffi_yttrium.so"
# file4="target/armv7-linux-androideabi/debug/libuniffi_yttrium.so"
# file_size1=$(stat -f%z $file1)
# file_size2=$(stat -f%z $file2)
# file_size3=$(stat -f%z $file3)
# file_size4=$(stat -f%z $file4)
# echo "debug,$file_size1,$file_size2,$file_size3,$file_size4" >> $output

# profiles="release uniffi-release uniffi-release-v2 profile1 profile2 profile21 profile22 profile3 profile4 profile5 profile6 profile7 profile8 profile9"
profiles="profile6 profile8 profile9 profile10"
for profile in $profiles; do
  echo "$profile"
  cargo build --profile $profile
  cargo ndk -t armeabi-v7a -t arm64-v8a build --profile=$profile --features=uniffi/cli
  file1="target/$profile/libuniffi_yttrium.a"
  file2="target/$profile/libuniffi_yttrium.dylib"
  file3="target/aarch64-linux-android/$profile/libuniffi_yttrium.so"
  file4="target/armv7-linux-androideabi/$profile/libuniffi_yttrium.so"
  file_size1=$(stat -f%z $file1)
  file_size2=$(stat -f%z $file2)
  file_size3=$(stat -f%z $file3)
  file_size4=$(stat -f%z $file4)
  echo "$profile,$file_size1,$file_size2,$file_size3,$file_size4" >> $output
done

# stdopt_profiles="uniffi-release-v2 profile6 profile7 profile8 profile9"
stdopt_profiles="profile6 profile8 profile9 profile10"
for profile in $stdopt_profiles; do
  echo "$profile"
  echo "build1"
  cargo +nightly build --profile $profile --target-dir="target/nightly"
  echo "build2"
  cargo +nightly ndk -t armeabi-v7a -t arm64-v8a build --profile=$profile --features=uniffi/cli --target-dir="target/nightly"
  file1="target/nightly/$profile/libuniffi_yttrium.a"
  file2="target/nightly/$profile/libuniffi_yttrium.dylib"
  file3="target/nightly/aarch64-linux-android/$profile/libuniffi_yttrium.so"
  file4="target/nightly/armv7-linux-androideabi/$profile/libuniffi_yttrium.so"
  file_size1=$(stat -f%z $file1)
  file_size2=$(stat -f%z $file2)
  file_size3=$(stat -f%z $file3)
  file_size4=$(stat -f%z $file4)
  echo "$profile-nightly,$file_size1,$file_size2,$file_size3,$file_size4" >> $output
  
  echo "build3"
  cargo +nightly build \
    -Z build-std=std,panic_abort \
    -Z build-std-features="optimize_for_size" \
    --target aarch64-apple-darwin --profile $profile --target-dir="target/nightly-stdopt"
  echo "build4"
  cargo +nightly ndk \
    -t armeabi-v7a -t arm64-v8a build --profile=$profile --features=uniffi/cli --target-dir="target/nightly-stdopt" \
    -Z build-std=std,panic_abort \
    -Z build-std-features="optimize_for_size"
  stdoptfile1="target/nightly-stdopt/aarch64-apple-darwin/$profile/libuniffi_yttrium.a"
  stdoptfile2="target/nightly-stdopt/aarch64-apple-darwin/$profile/libuniffi_yttrium.dylib"
  stdoptfile3="target/nightly-stdopt/aarch64-linux-android/$profile/libuniffi_yttrium.so"
  stdoptfile4="target/nightly-stdopt/armv7-linux-androideabi/$profile/libuniffi_yttrium.so"
  stdoptfile_size1=$(stat -f%z $stdoptfile1)
  stdoptfile_size2=$(stat -f%z $stdoptfile2)
  stdoptfile_size3=$(stat -f%z $stdoptfile3)
  stdoptfile_size4=$(stat -f%z $stdoptfile4)
  echo "$profile-nightly-stdopt,$stdoptfile_size1,$stdoptfile_size2,$stdoptfile_size3,$stdoptfile_size4" >> $output

  echo "build5"
  RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cargo +nightly build \
    -Z build-std=std,panic_abort \
    -Z build-std-features="optimize_for_size" \
    --target aarch64-apple-darwin --profile $profile --target-dir="target/nightly-stdopt-extra"
  echo "build6"
  RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cargo +nightly ndk \
    -t armeabi-v7a -t arm64-v8a build --profile=$profile --features=uniffi/cli --target-dir="target/nightly-stdopt-extra" \
    -Z build-std=std,panic_abort \
    -Z build-std-features="optimize_for_size"
  stdoptextrafile1="target/nightly-stdopt-extra/aarch64-apple-darwin/$profile/libuniffi_yttrium.a"
  stdoptextrafile2="target/nightly-stdopt-extra/aarch64-apple-darwin/$profile/libuniffi_yttrium.dylib"
  stdoptextrafile3="target/nightly-stdopt-extra/aarch64-linux-android/$profile/libuniffi_yttrium.so"
  stdoptextrafile4="target/nightly-stdopt-extra/armv7-linux-androideabi/$profile/libuniffi_yttrium.so"
  stdoptextrafile_size1=$(stat -f%z $stdoptextrafile1)
  stdoptextrafile_size2=$(stat -f%z $stdoptextrafile2)
  stdoptextrafile_size3=$(stat -f%z $stdoptextrafile3)
  stdoptextrafile_size4=$(stat -f%z $stdoptextrafile4)
  echo "$profile-nightly-stdopt-extra,$stdoptextrafile_size1,$stdoptextrafile_size2,$stdoptextrafile_size3,$stdoptextrafile_size4" >> $output
done

# extra2: adds panic_immediate_abort
stdopt_profiles="profile8 profile9 profile10"
for profile in $stdopt_profiles; do
  echo "build7"
  RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cargo +nightly ndk \
    -t armeabi-v7a -t arm64-v8a build --profile=$profile --features=uniffi/cli --target-dir="target/nightly-stdopt-extra2" \
    -Z build-std=std,panic_abort \
    -Z build-std-features="optimize_for_size,panic_immediate_abort"
  stdoptextra2file3="target/nightly-stdopt-extra2/aarch64-linux-android/$profile/libuniffi_yttrium.so"
  stdoptextra2file4="target/nightly-stdopt-extra2/armv7-linux-androideabi/$profile/libuniffi_yttrium.so"
  stdoptextra2file_size3=$(stat -f%z $stdoptextra2file3)
  stdoptextra2file_size4=$(stat -f%z $stdoptextra2file4)
  echo "$profile-nightly-stdopt-extra2,N/A,N/A,$stdoptextra2file_size3,$stdoptextra2file_size4" >> $output
done
