#!/bin/bash

cd docker
docker buildx build -t ghcr.io/gngpp/xunlei:$tag  --platform linux/amd64,linux/arm64 --push --build-arg VERSION=$tag --build-arg ARCH=x86_64 .
docker buildx build -t gngpp/xunlei:$tag --platform linux/amd64,linux/arm64 --push --build-arg VERSION=$tag --build-arg ARCH=x86_64 .
cd -