FROM alpine:latest as builder

ARG VERSION
ARG TARGETPLATFORM
ARG BUILDPLATFORM

RUN if [ "${TARGETPLATFORM}" = "linux/arm64" ]; then \
        echo "aarch64" > arch; \
    elif [ "${TARGETPLATFORM}" = "linux/amd64" ]; then \
        echo "x86_64" > arch; \
    fi
RUN apk update && apk add wget
RUN mkdir -p /var/packages/pan-xunlei-com/target
RUN wget https://github.com/gngpp/xunlei/releases/download/v${VERSION}/xunlei-launch-${VERSION}-$(cat arch)-unknown-linux-musl.tar.gz
RUN tar -xvf xunlei-launch-${VERSION}-$(cat arch)-unknown-linux-musl.tar.gz && chmod 755 /var/packages/pan-xunlei-com/target
RUN mv xunlei-launch-${VERSION}-$(cat arch)-unknown-linux-musl/bin/* /var/packages/pan-xunlei-com/target/
RUN mv xunlei-launch-${VERSION}-$(cat arch)-unknown-linux-musl/xunlei .

FROM alpine:latest
MAINTAINER gngpp <gngppz@gmail.com>
LABEL org.opencontainers.image.source https://github.com/gngpp/xunlei
LABEL name=xunlei
LABEL url=https://github.com/gngpp/xunlei
WORKDIR /rootfs

ENV LANG=C.UTF-8 DEBIAN_FRONTEND=noninteractive LANG=zh_CN.UTF-8 LANGUAGE=zh_CN.UTF-8 LC_ALL=C

RUN apk update && apk add bash

COPY run.sh /bin/run.sh
COPY --from=builder /xunlei /bin/xunlei
COPY --from=builder /var/packages/pan-xunlei-com/target /var/packages/pan-xunlei-com/target

VOLUME [ "/opt/data", "/downloads" ]
ENTRYPOINT ["/bin/run.sh"]