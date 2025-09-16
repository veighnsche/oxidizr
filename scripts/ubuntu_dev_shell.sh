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

echo "[dev-shell] Installing uutils coreutils via cargo (crates.io)..."
cargo install coreutils
if [ -x "$HOME/.cargo/bin/uutils" ]; then
  install -Dm0755 "$HOME/.cargo/bin/uutils" "/opt/uutils/uutils"
elif [ -x "$HOME/.cargo/bin/coreutils" ]; then
  install -Dm0755 "$HOME/.cargo/bin/coreutils" "/opt/uutils/uutils"
else
  echo "[dev-shell] ERROR: Neither uutils nor coreutils binary found in cargo bin" >&2
  ls -l "$HOME/.cargo/bin" || true
  exit 1
fi

# Install uutils-findutils (best effort); fallback to stub
echo "[dev-shell] Installing uutils-findutils via cargo (best effort)..."
if cargo install uutils-findutils >/dev/null 2>&1; then
  if [ -x "$HOME/.cargo/bin/uutils-findutils" ]; then
    install -Dm0755 "$HOME/.cargo/bin/uutils-findutils" "/opt/uutils-findutils/uutils-findutils"
  fi
else
  echo -e "#!/usr/bin/env bash\necho uutils-findutils-dev-shell-stub" > /opt/uutils-findutils/uutils-findutils
  chmod 0755 /opt/uutils-findutils/uutils-findutils
fi

# Try to fetch sudo-rs (skipped in dev shell)
echo "[dev-shell] sudo-rs fetch skipped in dev shell (optional)"

# Apply replacements under a fakeroot to avoid touching container live /
FROOT="/opt/fakeroot"
mkdir -p "$FROOT/usr/bin" "$FROOT/var/lock"

set -x
# Copy artifacts inside fakeroot so SafePath can validate sources
install -Dm0755 "/opt/uutils/uutils" "$FROOT/opt/uutils/uutils"
install -Dm0755 "/opt/uutils-findutils/uutils-findutils" "$FROOT/opt/uutils-findutils/uutils-findutils"

"$OXI" --root "$FROOT" --commit use coreutils --offline --use-local "$FROOT/opt/uutils/uutils" || { echo "failed to use coreutils" >&2; exit 1; }
"$OXI" --root "$FROOT" --commit use findutils --offline --use-local "$FROOT/opt/uutils-findutils/uutils-findutils" || { echo "failed to use findutils" >&2; exit 1; }
set +x

echo "\n[INFO] Replacements applied under fakeroot: $FROOT"
echo "[INFO] Launching interactive shell with PATH prefixed so \"ls\" resolves to uutils.\n"
echo "Commands to try:"
echo "  which ls && ls --version | head -n1"
echo "  readlink -f \"$(command -v ls)\""
echo

export PATH="$FROOT/usr/bin:$PATH"
exec bash --noprofile --norc
'
