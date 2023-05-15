#!/bin/bash

[ ! -d uploads ] && mkdir uploads
[ ! -d bin ] && mkdir bin

root=$(pwd)
target_list=(x86_64-unknown-linux-gnu x86_64-unknown-linux-musl aarch64-unknown-linux-gnu aarch64-unknown-linux-musl)
for target in ${target_list[@]}; do

  # default feature
  cargo zigbuild --release --target=$target
  upx target/$target/release/xunlei
  cargo deb --target=$target --no-build --no-strip
  cargo generate-rpm --target=$target --payload-compress none
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
  cd target/$target/generate-rpm
  rename 's/.*/xunlei-'$tag'-'$target'.rpm/' xunlei*.rpm
  mv ./* $root/uploads/
  cd -

  # embed feature
  if [[ $target == *"aarch64"* ]]; then
    arch=aarch64 bash +x unpack.sh
  else
    bash +x unpack.sh
  fi
  cargo zigbuild --release --target=$target --no-default-features --features embed
  upx target/$target/release/xunlei
  cargo deb --target=$target --no-build --no-strip
  cargo generate-rpm --target=$target --payload-compress none
  cd target/$target/release
  tar czvf xunlei-embed-$tag-$target.tar.gz xunlei
  shasum -a 256 xunlei-embed-$tag-$target.tar.gz >xunlei-embed-$tag-$target.tar.gz.sha256
  mv xunlei-embed-$tag-$target.tar.gz $root/uploads/
  mv xunlei-embed-$tag-$target.tar.gz.sha256 $root/uploads/
  cd -
  cd target/$target/debian
  rename 's/.*/xunlei-embed-'$tag'-'$target'.deb/' *.deb
  mv ./* $root/uploads/
  cd -
  cd target/$target/generate-rpm
  rename 's/.*/xunlei-embed-'$tag'-'$target'.rpm/' xunlei*.rpm
  mv ./* $root/uploads/
  cd -

  # launch feature
  mkdir -p xunlei-launch-$tag-$target/bin
  mv bin/* xunlei-launch-$tag-$target/bin/
  cargo zigbuild --release --target=$target --no-default-features --features launch
  mv target/$target/release/xunlei xunlei-launch-$tag-$target/
  tar -czvf xunlei-launch-$tag-$target.tar.gz xunlei-launch-$tag-$target/*
  shasum -a 256 xunlei-launch-$tag-$target.tar.gz >xunlei-launch-$tag-$target.tar.gz.sha256
  mv xunlei-launch-$tag-$target.tar.gz uploads/
  mv xunlei-launch-$tag-$target.tar.gz.sha256 uploads/

  rm -r bin/*
  ls -lah uploads
done
