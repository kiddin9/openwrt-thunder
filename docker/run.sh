#!/bin/bash

target_path=/var/packages/pan-xunlei-com/target
version=$(cat $target_path/version)
arch=$(cat /arch)

if strings -a $target_path/xunlei-pan-cli-web | grep -q UPX; then
    upx -d $target_path/xunlei-pan-cli-web >/dev/null
fi

if strings -a $target_path/xunlei-pan-cli.$version.$arch | grep -q UPX; then
    upx -d $target_path/xunlei-pan-cli.$version.$arch >/dev/null
fi

mkdir -p /rootfs/bin /rootfs/run \
    /rootfs/lib /rootfs/proc \
    /rootfs/usr /rootfs/mnt \
    /rootfs/etc /rootfs/sbin \
    /rootfs/sys /rootfs/dev \
    /rootfs/var /rootfs/tmp \
    /rootfs/root /rootfs/boot \
    /rootfs/opt/data /rootfs/downloads \
    /opt/data /downloads

target_var=/var/packages/pan-xunlei-com/target/var
if [ ! -d "$target_var" ]; then
    path="/var/packages/pan-xunlei-com/target/host/etc"
    mkdir -p $path
    id=$(cat /proc/sys/kernel/random/uuid | cut -c1-7)
    echo "unique=\"synology_${id}_720+\"" >$path/synoinfo.conf
    chmod 755 $path/synoinfo.conf

    path="/var/packages/pan-xunlei-com/target/host/usr/syno/synoman/webman/modules"
    mkdir -p $path
    echo -e '#!/usr/bin/env sh\necho OK' >$path/authenticate.cgi
    chmod 755 $path/authenticate.cgi
else
    find $target_var -type f \( -name '*.pid' -o -name '*.pid.child' \) -delete
    find $target_var -type s -name '*.sock' -delete
fi

if [ -f /etc/synoinfo.conf ]; then
    rm /etc/synoinfo.conf
fi

if [ -f /usr/syno/synoman/webman/modules/authenticate.cgi ]; then
    rm /usr/syno/synoman/webman/modules/authenticate.cgi
fi

mkdir -p /usr/syno/synoman/webman/modules
ln -s /var/packages/pan-xunlei-com/target/host/etc/synoinfo.conf /etc/synoinfo.conf
ln -s /var/packages/pan-xunlei-com/target/host/usr/syno/synoman/webman/modules/authenticate.cgi /usr/syno/synoman/webman/modules/authenticate.cgi

dir_list=(/bin /run /lib /usr /mnt /etc /sbin /dev /var /tmp /root /proc /opt/data /downloads)
for dir in ${dir_list[@]}; do
    mount --bind $dir /rootfs$dir
done

chroot /rootfs /bin/bash -c "echo 'nameserver 119.29.29.29' > /etc/resolv.conf && /bin/mount -t proc none /proc && /bin/xunlei launch -c /opt/data -d /downloads"

exec "$@"
