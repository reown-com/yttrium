#!/bin/bash
set -eo pipefail

# Pre-ran output can be found here and is updated periodically:
# https://docs.google.com/spreadsheets/d/1Ko7TmIbNrgB2PWN8cPDM4GL6Iy2d8c-VrD0FnQ9SicI/edit?usp=sharing

output="benchmark-profile-output-sizes.csv"
echo "profile,libuniffi_yttrium.a,libuniffi_yttrium.dylib" > $output

echo "debug"
cargo build
file1="target/debug/libuniffi_yttrium.a"
file2="target/debug/libuniffi_yttrium.dylib"
file_size1=$(stat -f%z $file1)
file_size2=$(stat -f%z $file2)
echo "debug,$file_size1,$file_size2" >> $output

profiles="release uniffi-release profile1 profile2 profile21 profile22 profile3 profile4 profile5 profile6 profile7 profile8 profile9"
for profile in $profiles; do
  echo "$profile"
  cargo build --profile $profile
  file1="target/$profile/libuniffi_yttrium.a"
  file2="target/$profile/libuniffi_yttrium.dylib"
  file_size1=$(stat -f%z $file1)
  file_size2=$(stat -f%z $file2)
  echo "$profile,$file_size1,$file_size2" >> $output
done

stdopt_profiles="profile6 profile7 profile8 profile9"
for profile in $stdopt_profiles; do
  echo "$profile"
  cargo +nightly build --profile $profile --target-dir="target/nightly"
  file1="target/nightly/$profile/libuniffi_yttrium.a"
  file2="target/nightly/$profile/libuniffi_yttrium.dylib"
  file_size1=$(stat -f%z $file1)
  file_size2=$(stat -f%z $file2)
  echo "$profile-nightly,$file_size1,$file_size2" >> $output

  cargo +nightly build \
    -Z build-std=std,panic_abort \
    -Z build-std-features="optimize_for_size" \
    --target aarch64-apple-darwin --profile $profile --target-dir="target/nightly-stdopt"
  stdoptfile1="target/nightly-stdopt/aarch64-apple-darwin/$profile/libuniffi_yttrium.a"
  stdoptfile2="target/nightly-stdopt/aarch64-apple-darwin/$profile/libuniffi_yttrium.dylib"
  stdoptfile_size1=$(stat -f%z $stdoptfile1)
  stdoptfile_size2=$(stat -f%z $stdoptfile2)
  echo "$profile-nightly-stdopt,$stdoptfile_size1,$stdoptfile_size2" >> $output

  RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cargo +nightly build \
    -Z build-std=std,panic_abort \
    -Z build-std-features="optimize_for_size" \
    --target aarch64-apple-darwin --profile $profile --target-dir="target/nightly-stdopt-extra"
  stdoptextrafile1="target/nightly-stdopt-extra/aarch64-apple-darwin/$profile/libuniffi_yttrium.a"
  stdoptextrafile2="target/nightly-stdopt-extra/aarch64-apple-darwin/$profile/libuniffi_yttrium.dylib"
  stdoptextrafile_size1=$(stat -f%z $stdoptextrafile1)
  stdoptextrafile_size2=$(stat -f%z $stdoptextrafile2)
  echo "$profile-nightly-stdopt-extra,$stdoptextrafile_size1,$stdoptextrafile_size2" >> $output
done
