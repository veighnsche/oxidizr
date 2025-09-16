#!/usr/bin/env bash
set -euo pipefail

# Interactive Ubuntu shell with oxidizr-deb replacements applied (use --commit).
# Safe for your host: everything happens inside a disposable Ubuntu container.
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
apt-get install -y curl ca-certificates build-essential pkg-config git
curl https://sh.rustup.rs -sSf | sh -s -- -y
. $HOME/.cargo/env

# Build oxidizr-deb
cargo build -p oxidizr-deb
OXI="target/debug/oxidizr-deb"

mkdir -p /opt/uutils /opt/uutils-findutils /opt/sudo-rs

# Install uutils-coreutils via cargo
echo "[dev-shell] Installing uutils-coreutils via cargo..."
cargo install --locked --git https://github.com/uutils/coreutils uutils
install -Dm0755 "$HOME/.cargo/bin/uutils" "/opt/uutils/uutils"

# Install uutils-findutils (best effort); fallback to stub
echo "[dev-shell] Installing uutils-findutils via cargo (best effort)..."
if cargo install --locked --git https://github.com/uutils/findutils uutils-findutils >/dev/null 2>&1; then
  install -Dm0755 "$HOME/.cargo/bin/uutils-findutils" "/opt/uutils-findutils/uutils-findutils"
else
  echo -e "#!/usr/bin/env bash\necho uutils-findutils-dev-shell-stub" > /opt/uutils-findutils/uutils-findutils
  chmod 0755 /opt/uutils-findutils/uutils-findutils
fi

# Try to fetch sudo-rs (skipped in dev shell)
echo "[dev-shell] sudo-rs fetch skipped in dev shell (optional)"

# Apply replacements on the container live root using offline artifacts
set -x
"$OXI" --commit use coreutils --offline --use-local /opt/uutils/uutils || { echo "failed to use coreutils" >&2; exit 1; }
"$OXI" --commit use findutils --offline --use-local /opt/uutils-findutils/uutils-findutils || { echo "failed to use findutils" >&2; exit 1; }
set +x

echo "\n[INFO] Replacements applied. You are now in an interactive shell.\n"
echo "Commands to try:"
echo "  which ls && ls --version | head -n1"
echo "  which find && find --version 2>/dev/null || echo find-version-may-not-print-run-find-help"
echo "  which sudo && sudo --version 2>/dev/null || echo sudo-may-require-setuid-root-4755"
echo

exec bash
'
