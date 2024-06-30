#!/bin/bash

set -e

[ ! -d uploads ] && mkdir uploads

root=$(pwd)
target_list=(x86_64-unknown-linux-musl aarch64-unknown-linux-musl)
for target in ${target_list[@]}; do

  rustup target add $target
  
  # default feature
  cargo zigbuild --release --target=$target
  upx --lzma target/$target/release/thunder
  cargo deb --target=$target --no-build --no-strip
  cd target/$target/release
  tar czvf thunder-$tag-$target.tar.gz thunder
  shasum -a 256 thunder-$tag-$target.tar.gz >thunder-$tag-$target.tar.gz.sha256
  mv thunder-$tag-$target.tar.gz $root/uploads/
  mv thunder-$tag-$target.tar.gz.sha256 $root/uploads/
  cd -
  cd target/$target/debian
  rename 's/.*/thunder-'$tag'-'$target'.deb/' *.deb
  mv ./* $root/uploads/
  cd -

  ls -lah uploads
done
