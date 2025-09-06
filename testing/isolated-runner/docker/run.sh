#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

IMAGE_TAG="oxidizr-arch:latest"

echo "[+] Building Docker image: $IMAGE_TAG"
docker build -t "$IMAGE_TAG" .

# Run with repo mounted at /workspace so scripts can copy into /root/project
ROOT_DIR="$(git rev-parse --show-toplevel 2>/dev/null || realpath ../.. | sed 's#/$##')"
echo "[+] Starting container from $IMAGE_TAG with mount: $ROOT_DIR -> /workspace"
docker run --rm -it \
  -v "$ROOT_DIR:/workspace" \
  --name oxidizr-arch-test \
  "$IMAGE_TAG" \
  "/workspace/rust_coreutils_switch/testing/isolated-runner/docker/entrypoint.sh"
