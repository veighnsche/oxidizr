#!/usr/bin/env bash
set -euo pipefail
set -x

# Non-interactive proof: disposable Ubuntu container, apply replacements on live root
# using the CLI with APT/DPKG first and CLI fallback online when packages are unavailable.

IMG="ubuntu:24.04"
WORKDIR="/work"

docker run --rm -i -v "$(pwd)":${WORKDIR} -w ${WORKDIR} ${IMG} bash -s <<'SCRIPT'
set -euo pipefail
apt-get update
apt-get install -y curl ca-certificates build-essential pkg-config git
curl https://sh.rustup.rs -sSf | sh -s -- -y
. "$HOME/.cargo/env"

# Build oxidizr-deb
cargo build -p oxidizr-deb
OXI="target/debug/oxidizr-deb"

# Apply replacements on live root via CLI (APT first, online fallback if needed)
"$OXI" --assume-yes --commit use coreutils
"$OXI" --assume-yes --commit use findutils

# Proof: show link target and version
set +e
which ls || true
LS_BIN=$(which ls || true)
LS_TARGET=$(readlink -f "$LS_BIN" || true)
LS_VERSION=$(ls --version 2>&1 | head -n2 || true)
set -e

echo "[PROOF] which ls: ${LS_BIN}"
echo "[PROOF] ls resolves to -> ${LS_TARGET}"
echo "[PROOF] ls --version:\n${LS_VERSION}"

# Additional diagnostics
command -v ls || true
stat -c '%N' /usr/bin/ls || true

if ! echo "${LS_VERSION}" | grep -iq "uutils"; then
  echo "[proof] ls --version does not mention uutils" >&2
  exit 1
fi
if [ ! -L "/usr/bin/ls" ]; then
  echo "[proof] /usr/bin/ls is not a symlink; expected symlink into replacement" >&2
  exit 1
fi
if ! readlink -f /usr/bin/ls | grep -q "/opt/oxidizr/replacements"; then
  echo "[proof] Unexpected ls target: ${LS_TARGET}" >&2
  exit 1
fi

echo "[OK] uutils-coreutils is active."
SCRIPT
