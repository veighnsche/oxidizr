#!/usr/bin/env bash
set -euo pipefail

# Interactive Ubuntu shell with oxidizr-deb replacements applied on the live root inside the container.
# Safe for your host: everything happens inside a disposable Ubuntu container. CLI uses APT first, then online fallback.
# Requirements: Docker installed on the host.
# Usage: bash scripts/ubuntu_dev_shell.sh

IMG="ubuntu:24.04"
WORKDIR="/work"

exec docker run --rm -it \
  -v "$(pwd)":${WORKDIR} \
  -w ${WORKDIR} \
  ${IMG} bash -lc '
set -euo pipefail
apt-get update
apt-get install -y curl ca-certificates build-essential pkg-config git xz-utils
curl https://sh.rustup.rs -sSf | sh -s -- -y
. $HOME/.cargo/env

## Build oxidizr-deb (use container-local target dir to avoid host perms)
export CARGO_TARGET_DIR="${WORKDIR}/.target-in-container"
cargo build -p oxidizr-deb
OXI="target/debug/oxidizr-deb"

set -x
"$OXI" --assume-yes --commit use coreutils || { echo "failed to use coreutils" >&2; exit 1; }
"$OXI" --assume-yes --commit use findutils || { echo "failed to use findutils" >&2; exit 1; }
set +x

echo "\n[INFO] Replacements applied on live root (container). Dropping into interactive shell.\n"
echo "Commands to try:"
echo "  which ls && ls --version | head -n1"
echo "  readlink -f /usr/bin/ls"
echo "  which find && (find --version 2>/dev/null || find --help | head -n3)"
echo

exec bash --noprofile --norc
'
