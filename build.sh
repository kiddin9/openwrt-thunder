#!/bin/bash

set -e

[ ! -d uploads ] && mkdir uploads

root=$(pwd)
target_list=(x86_64-unknown-linux-musl aarch64-unknown-linux-musl)
for target in ${target_list[@]}; do

  # default feature
  cargo zigbuild --release --target=$target
  upx --lzma target/$target/release/xunlei
  cargo deb --target=$target --no-build --no-strip
  cd target/$target/release
  tar czvf xunlei-$tag-$target.tar.gz xunlei
  shasum -a 256 xunlei-$tag-$target.tar.gz >xunlei-$tag-$target.tar.gz.sha256
  mv xunlei-$tag-$target.tar.gz $root/uploads/
  mv xunlei-$tag-$target.tar.gz.sha256 $root/uploads/
  cd -
  cd target/$target/debian
  rename 's/.*/xunlei-'$tag'-'$target'.deb/' *.deb
  mv ./* $root/uploads/
  cd -

  ls -lah uploads
done