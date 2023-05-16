FROM alpine:latest as builder

ARG VERSION
ARG ARCH

RUN apk update && apk add wget
RUN mkdir -p /var/packages/pan-xunlei-com/target
RUN wget https://github.com/gngpp/xunlei/releases/download/v${VERSION}/xunlei-launch-${VERSION}-${ARCH}-unknown-linux-musl.tar.gz
RUN tar -xvf xunlei-launch-${VERSION}-${ARCH}-unknown-linux-musl.tar.gz && chmod 755 /var/packages/pan-xunlei-com/target
RUN mv xunlei-launch-${VERSION}-${ARCH}-unknown-linux-musl/bin/* /var/packages/pan-xunlei-com/target/
RUN mv xunlei-launch-${VERSION}-${ARCH}-unknown-linux-musl/xunlei .
COPY authenticate.cgi /var/packages/pan-xunlei-com/target/host/usr/syno/synoman/webman/modules/authenticate.cgi
COPY synoinfo.conf /var/packages/pan-xunlei-com/target/host/etc/synoinfo.conf
RUN chmod 755 /var/packages/pan-xunlei-com/target/host/usr/syno/synoman/webman/modules/authenticate.cgi
RUN chmod 755 /var/packages/pan-xunlei-com/target/host/etc/synoinfo.conf

FROM alpine:latest
LABEL org.opencontainers.image.source https://github.com/gngpp/xunlei
WORKDIR /rootfs

ENV LANG=C.UTF-8 DEBIAN_FRONTEND=noninteractive LANG=zh_CN.UTF-8 LANGUAGE=zh_CN.UTF-8 LC_ALL=C

RUN apk update && apk add bash
RUN mkdir /rootfs/bin /rootfs/run \
    /rootfs/lib /rootfs/proc \
    /rootfs/usr /rootfs/mnt \
    /rootfs/etc /rootfs/sbin \
    /rootfs/sys /rootfs/dev \
    /rootfs/var /rootfs/tmp \
    /rootfs/root /rootfs/boot \
    /opt/xunlei/downloads /opt/xunlei && \
    mkdir -p /rootfs/opt/xunlei && \
    mkdir -p /rootfs/opt/xunlei/downloads

COPY run_script.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/run_script.sh

COPY --from=builder /var/packages/pan-xunlei-com/target /var/packages/pan-xunlei-com/target
COPY --from=builder /xunlei /usr/bin/xunlei

VOLUME [ "/opt/xunlei", "/opt/xunlei/downloads" ]
ENTRYPOINT ["/usr/local/bin/run_script.sh"]