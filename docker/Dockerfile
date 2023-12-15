FROM alpine:latest as builder

ARG VERSION
ARG TARGETPLATFORM
ARG BUILDPLATFORM

RUN if [ "${TARGETPLATFORM}" = "linux/arm64" ]; then \
        echo "aarch64" > arch; \
        echo "arm64" > ali_arch; \
    elif [ "${TARGETPLATFORM}" = "linux/amd64" ]; then \
        echo "x86_64" > arch; \
        echo "amd64" > ali_arch; \
    fi
RUN apk update && apk add wget upx
RUN mkdir -p /var/packages/pan-xunlei-com/target && chmod 755 /var/packages/pan-xunlei-com/target
RUN wget https://github.com/gngpp/thunder/releases/download/v${VERSION}/xunlei-${VERSION}-$(cat arch)-unknown-linux-musl.tar.gz
RUN tar -xvf thunder-${VERSION}-$(cat arch)-unknown-linux-musl.tar.gz
RUN mv thunder-${VERSION}-$(cat arch)-unknown-linux-musl/bin/* /var/packages/pan-xunlei-com/target/
RUN mv thunder-${VERSION}-$(cat arch)-unknown-linux-musl/thunder .
RUN upx --lzma /var/packages/pan-xunlei-com/target/xunlei-pan-cli-web
RUN upx --lzma /var/packages/pan-xunlei-com/target/xunlei-pan-cli-launcher.$(cat ali_arch)
RUN upx --lzma /var/packages/pan-xunlei-com/target/xunlei-pan-cli.$(cat /var/packages/pan-xunlei-com/target/version).$(cat ali_arch)

FROM alpine:latest
LABEL org.opencontainers.image.authors "gngpp <gngppz@gmail.com>"
LABEL org.opencontainers.image.source https://github.com/gngpp/thunder
LABEL name thunder
LABEL url https://github.com/gngpp/thunder
WORKDIR /rootfs

ARG VERSION
ARG TARGETPLATFORM
ARG BUILDPLATFORM
ENV LANG=C.UTF-8 DEBIAN_FRONTEND=noninteractive LANG=zh_CN.UTF-8 LANGUAGE=zh_CN.UTF-8 LC_ALL=C

RUN apk update && apk add bash upx

COPY run.sh /bin/run.sh
COPY --from=builder /ali_arch /arch
COPY --from=builder /thunder /bin/thunder
COPY --from=builder /var/packages/pan-xunlei-com/target /var/packages/pan-xunlei-com/target

RUN mkdir -p /rootfs/bin /rootfs/run \
    /rootfs/lib /rootfs/proc \
    /rootfs/usr /rootfs/mnt \
    /rootfs/etc /rootfs/sbin \
    /rootfs/sys /rootfs/dev \
    /rootfs/var /rootfs/tmp \
    /rootfs/root /rootfs/boot \
    /rootfs/opt/data /rootfs/downloads /rootfs/thunder \
    /opt/data /downloads /thunder

VOLUME [ "/opt/data", "/downloads" ]
ENTRYPOINT ["/bin/bash", "+x", "/bin/run.sh"]