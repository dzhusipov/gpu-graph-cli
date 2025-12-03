#!/bin/bash

set -e

IMAGE_NAME="gpu-graph-cli"
CONTAINER_NAME="gpu-graph-cli-container"

echo "=== Stopping and removing existing container (if exists) ==="
docker stop $CONTAINER_NAME 2>/dev/null || true
docker rm $CONTAINER_NAME 2>/dev/null || true

echo "=== Removing existing image (if exists) ==="
docker rmi $IMAGE_NAME 2>/dev/null || true

echo "=== Building Docker image ==="
docker build -t $IMAGE_NAME .

echo "=== Running container ==="
docker run --gpus all -it --rm --name $CONTAINER_NAME $IMAGE_NAME

