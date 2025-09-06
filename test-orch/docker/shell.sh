#!/usr/bin/env bash
set -euo pipefail

IMAGE_TAG="${IMAGE_TAG:-oxidizr-arch:latest}"
DOCKER_CTX_REL="test-orch/docker"

# Resolve repo root
if ROOT_DIR=$(git rev-parse --show-toplevel 2>/dev/null); then
  :
else
  # Fallback to script-relative two directories up
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
fi

# Build image if missing
if ! docker image inspect "$IMAGE_TAG" >/dev/null 2>&1; then
  echo "[shell] Building image $IMAGE_TAG from $DOCKER_CTX_REL..."
  docker build -t "$IMAGE_TAG" "$ROOT_DIR/$DOCKER_CTX_REL"
fi

echo "[shell] Starting interactive container with repo mounted at /workspace"
echo "[shell] Image: $IMAGE_TAG"
echo "[shell] Host mount: $ROOT_DIR -> /workspace"

exec docker run --rm -it \
  -v "$ROOT_DIR:/workspace" \
  --name oxidizr-arch-dev \
  -w /workspace \
  "$IMAGE_TAG" \
  bash -l
