#!/bin/bash

cd docker
docker buildx build --tag ghcr.io/gngpp/xunlei:$tag --tag gngpp/xunlei:$tag --platform linux/amd64,linux/arm64 --push --build-arg VERSION=$tag .
cd -