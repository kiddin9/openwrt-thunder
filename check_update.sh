#!/bin/bash
bash +x unpack.sh x86_64
current_version="$(cat Cargo.toml | grep version | head -1 | awk -F= '{gsub(/[[:space:]"\/]/,"",$2); print substr($2, 1, index($2,"-")-1)}')"
new_version="$(cat bin/version)"

echo "current_version=$current_version"
echo "new_version=$new_version"

export current_version=$current_version
export new_version=$new_version

sed -i 's/'$current_version'/'$new_version'/g' Cargo.toml
sed -i 's/'$current_version'/'$new_version'/g' openwrt/xunlei/Makefile
sed -i 's/'$current_version'/'$new_version'/g' README.md