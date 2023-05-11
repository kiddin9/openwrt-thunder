#!/bin/bash

: ${arch:="$(uname -m)"}

dir=$(pwd)/bin;
if ! [ -d "$dir" ]; then
    mkdir $dir  
fi

download_dir=${pwd}/tmp;
if ! [ -d "$download_dir" ]; then
    mkdir $download_dir  
fi

if [ "$arch" = "aarch64" ]; then
 arch=armv8; 
else
 arch=$(uname -m);
fi

filename="nasxunlei-DSM7-${arch}.spk"

cd $download_dir
if [ ! -f "$filename" ];then
    wget https://github.com/gngpp/nas-xunlei/releases/download/spk/$filename
fi
cp $download_dir/$filename $dir/$filename
cd -

cd $dir
tar --wildcards -Oxf $(find . -type f -name \*-${arch}.spk | head -n1) package.tgz | tar --wildcards -xJC ${dir} 'bin/bin/*' 'ui/index.cgi'
mv ${dir}/bin/bin/* ${dir}/
mv ${dir}/ui/index.cgi ${dir}/xunlei-pan-cli-web
rm -rf ${dir}/bin/bin
rm -rf ${dir}/bin
rm -rf ${dir}/ui
rm -f ${dir}/version_code ${dir}/*.spk
cd -
