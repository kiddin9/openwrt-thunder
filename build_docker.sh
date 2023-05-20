#!/bin/bash

cd docker
docker buildx build --platform linux/amd64,linux/arm64 \
    --tag ghcr.io/gngpp/xunlei:$tag \
    --tag gngpp/xunlei:$tag \
    --tag gngpp/xunlei:latest \
    --tag ghcr.io/gngpp/xunlei:latest \
    --build-arg VERSION=$tag --push .
cd -