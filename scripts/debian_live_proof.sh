#!/usr/bin/env bash
set -euo pipefail

# Live-root proof in a disposable Debian-family container.
# Demonstrates that the CLI itself performs:
# - APT/DPKG installation when available
# - Fallback online retrieval (cargo/github) when APT packages are unavailable
# - Safe atomic swaps via Switchyard
# - Optional package removal for findutils via apt-get purge
# Host is never mutated.

IMG="ubuntu:24.04"
WORKDIR="/work"

docker run --rm -i \
  -v "$(pwd)":${WORKDIR} \
  -w ${WORKDIR} \
  ${IMG} bash -s <<'SCRIPT'
set -euo pipefail
apt-get update
apt-get install -y curl ca-certificates build-essential pkg-config git
curl https://sh.rustup.rs -sSf | sh -s -- -y
. "$HOME/.cargo/env"

# Build CLI
cargo build -p oxidizr-deb
OXI="target/debug/oxidizr-deb"

# 1) use coreutils on live root (/) with CLI-driven fetch (APT first, fallback online if needed)
"$OXI" --assume-yes --commit use coreutils

# Proof for coreutils
echo "[proof] coreutils: which ls; readlink target; version"
command -v ls
readlink -f /usr/bin/ls || true
ls --version | head -n1

# 2) use findutils on live root with CLI-driven fetch
"$OXI" --assume-yes --commit use findutils

echo "[proof] findutils: which find; readlink target; version"
command -v find
readlink -f /usr/bin/find || true
find --version 2>/dev/null || echo "find --version may not print; running find --help" && find --help | head -n3 || true

# 3) replace findutils (purge distro) under guardrails
"$OXI" --assume-yes --commit replace findutils

echo "[proof] after replace: which find; readlink target"
command -v find
readlink -f /usr/bin/find || true

# Done
SCRIPT
