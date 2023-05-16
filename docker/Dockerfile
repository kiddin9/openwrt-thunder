FROM ubuntu:latest as builder

ARG VERSION
ARG ARCH

RUN apt update && apt install wget -y
RUN mkdir -p /var/packages/pan-xunlei-com/target
RUN wget https://github.com/gngpp/xunlei/releases/download/v${VERSION}/xunlei-launch-${VERSION}-${ARCH}-unknown-linux-gnu.tar.gz
RUN tar -xvf xunlei-launch-${VERSION}-${ARCH}-unknown-linux-gnu.tar.gz && chmod 755 /var/packages/pan-xunlei-com/target
RUN mv xunlei-launch-${VERSION}-${ARCH}-unknown-linux-gnu/bin/* /var/packages/pan-xunlei-com/target/
RUN mv xunlei-launch-${VERSION}-${ARCH}-unknown-linux-gnu/xunlei .
COPY authenticate.cgi /var/packages/pan-xunlei-com/target/host/usr/syno/synoman/webman/modules/authenticate.cgi
COPY synoinfo.conf /var/packages/pan-xunlei-com/target/host/etc/synoinfo.conf
RUN chmod 755 /var/packages/pan-xunlei-com/target/host/usr/syno/synoman/webman/modules/authenticate.cgi
RUN chmod 755 /var/packages/pan-xunlei-com/target/host/etc/synoinfo.conf

FROM ubuntu:latest
LABEL org.opencontainers.image.source https://github.com/gngpp/xunlei
WORKDIR /rootfs

ENV LANG=C.UTF-8 DEBIAN_FRONTEND=noninteractive LANG=zh_CN.UTF-8 LANGUAGE=zh_CN.UTF-8 LC_ALL=C

RUN sed -i 's/archive.ubuntu.com/mirrors.bfsu.edu.cn/g' /etc/apt/sources.list \ 
    && sed -i 's/security.ubuntu.com/mirrors.bfsu.edu.cn/g' /etc/apt/sources.list \
    && sed -i 's/ports.ubuntu.com/mirrors.bfsu.edu.cn/g' /etc/apt/sources.list \
    && apt-get update && apt-get -y --no-install-recommends install tzdata locales xfonts-wqy wget ca-certificates && rm -rf /var/lib/apt/lists/*

RUN localedef -i zh_CN -c -f UTF-8 -A /usr/share/locale/locale.alias zh_CN.UTF-8 && locale-gen zh_CN.UTF-8 && \
    echo "Asia/Shanghai" > /etc/timezone && \
    dpkg-reconfigure -f noninteractive tzdata && \
    rm -f /etc/localtime && cp /usr/share/zoneinfo/Asia/Shanghai /etc/localtime && \
    find /var/lib/apt/lists -type f -delete && \
    find /var/cache -type f -delete

RUN mkdir /rootfs/bin /rootfs/run \
    /rootfs/lib /rootfs/lib64 \
    /rootfs/lib32 /rootfs/libx32 \
    /rootfs/usr /rootfs/mnt \
    /rootfs/etc /rootfs/sbin \
    /rootfs/sys /rootfs/dev \
    /rootfs/var /rootfs/tmp \
    /rootfs/root /rootfs/boot \
    /rootfs/proc /opt/xunlei \
    /opt/xunlei/downloads && \
    mkdir -p /rootfs/opt/xunlei && \
    mkdir -p /rootfs/opt/xunlei/downloads

COPY run_script.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/run_script.sh

COPY --from=builder /var/packages/pan-xunlei-com/target /var/packages/pan-xunlei-com/target
COPY --from=builder /xunlei /usr/bin/xunlei

VOLUME [ "/opt/xunlei", "/opt/xunlei/downloads" ]
ENTRYPOINT ["/usr/local/bin/run_script.sh"]