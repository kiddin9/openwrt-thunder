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

if [ ! -f "/var/packages/pan-xunlei-com/target/host/etc/synoinfo.conf" ]; then
    path="/var/packages/pan-xunlei-com/target/host/usr/syno/synoman/webman/modules"
    mkdir -p $path
    echo -e '#!/usr/bin/env sh\necho OK' >$path/authenticate.cgi
    chmod 755 $path/authenticate.cgi
fi

if [ ! -f "/var/packages/pan-xunlei-com/target/host/etc/synoinfo.conf" ]; then
    path="/var/packages/pan-xunlei-com/target/host/etc"
    mkdir -p /var/packages/pan-xunlei-com/target/host/etc
    echo 'unique="synology_bb633c4_720+"' >$path/synoinfo.conf
    chmod 755 $path/synoinfo.conf
fi

if [ ! -d "/var/packages/pan-xunlei-com/target/var" ]; then
    id=$(cat /proc/sys/kernel/random/uuid | cut -c1-7)
    echo "unique=\"synology_${id}_720+\"" >/var/packages/pan-xunlei-com/target/host/etc/synoinfo.conf
fi

mkdir -p /usr/syno/synoman/webman/modules
ln -s /var/packages/pan-xunlei-com/target/host/etc/synoinfo.conf /etc/synoinfo.conf
ln -s /var/packages/pan-xunlei-com/target/host/usr/syno/synoman/webman/modules/authenticate.cgi /usr/syno/synoman/webman/modules/authenticate.cgi

umount /rootfs/bin > /dev/null
umount /rootfs/run > /dev/null
umount /rootfs/lib > /dev/null
umount /rootfs/usr > /dev/null
umount /rootfs/mnt > /dev/null
umount /rootfs/etc > /dev/null
umount /rootfs/sbin > /dev/null
umount /rootfs/dev > /dev/null
umount /rootfs/var > /dev/null
umount /rootfs/tmp > /dev/null
umount /rootfs/root > /dev/null
umount /rootfs/proc > /dev/null
umount /rootfs/opt/data > /dev/null
umount /rootfs/downloads > /dev/null

mount --bind /bin /rootfs/bin
mount --bind /run /rootfs/run
mount --bind /lib /rootfs/lib
mount --bind /usr /rootfs/usr
mount --bind /mnt /rootfs/mnt
mount --bind /etc /rootfs/etc
mount --bind /sbin /rootfs/sbin
mount --bind /dev /rootfs/dev
mount --bind /var /rootfs/var
mount --bind /tmp /rootfs/tmp
mount --bind /root /rootfs/root
mount --bind /proc /rootfs/proc
mount --bind /opt/data /rootfs/opt/data
mount --bind /downloads /rootfs/downloads

chroot /rootfs /bin/bash -c "echo 'nameserver 119.29.29.29' > /etc/resolv.conf && /bin/mount -t proc none /proc && /bin/xunlei launch -c /opt/data -d /downloads"

exec "$@"
