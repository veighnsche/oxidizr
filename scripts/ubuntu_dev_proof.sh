#!/usr/bin/env bash
set -euo pipefail
set -x

# Non-interactive proof: disposable Ubuntu container, apply replacements
# (offline artifacts) and print version/proof that uutils is active.

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

mkdir -p /opt/uutils /opt/uutils-findutils

# Build uutils-coreutils from crates.io and copy unified binary location
echo "[proof] Installing coreutils crate via cargo..."
cargo install coreutils
if [ -x "$HOME/.cargo/bin/uutils" ]; then
  install -Dm0755 "$HOME/.cargo/bin/uutils" "/opt/uutils/uutils"
elif [ -x "$HOME/.cargo/bin/coreutils" ]; then
  install -Dm0755 "$HOME/.cargo/bin/coreutils" "/opt/uutils/uutils"
else
  echo "[proof] Neither uutils nor coreutils binary found in cargo bin" >&2
  ls -l "$HOME/.cargo/bin" || true
  exit 1
fi

# Provide a minimal uutils-findutils stub
echo -e "#!/usr/bin/env bash\necho uutils-findutils-dev-shell-stub" > /opt/uutils-findutils/uutils-findutils
chmod 0755 /opt/uutils-findutils/uutils-findutils

FROOT="/opt/fakeroot"
mkdir -p "$FROOT/usr/bin" "$FROOT/var/lock"
# Copy artifacts inside fakeroot so SafePath can validate sources
install -Dm0755 "/opt/uutils/uutils" "$FROOT/opt/uutils/uutils"
install -Dm0755 "/opt/uutils-findutils/uutils-findutils" "$FROOT/opt/uutils-findutils/uutils-findutils"

# Apply replacements under fakeroot
"$OXI" --root "$FROOT" --commit use coreutils --offline --use-local "$FROOT/opt/uutils/uutils"
"$OXI" --root "$FROOT" --commit use findutils --offline --use-local "$FROOT/opt/uutils-findutils/uutils-findutils"

# Proof: show link target and version
set +e
PATH="$FROOT/usr/bin:$PATH" which ls || true
PATH="$FROOT/usr/bin:$PATH" ls --version 2>&1 | head -n2 || true
LS_BIN=$(PATH="$FROOT/usr/bin:$PATH" which ls || true)
LS_TARGET=$(readlink -f "$LS_BIN" || true)
LS_VERSION=$(PATH="$FROOT/usr/bin:$PATH" ls --version 2>&1 | head -n2 || true)
ls -l "$FROOT/usr/bin" || true
set -e

echo "[PROOF] which ls (with PATH prefixed): ${LS_BIN}"
echo "[PROOF] ls resolves to -> ${LS_TARGET}"
echo "[PROOF] ls --version (with PATH prefixed):\n${LS_VERSION}"

# Additional diagnostics
command -v ls || true
stat -c '%N' /usr/bin/ls || true
/opt/uutils/uutils --version 2>&1 | head -n2 || true

if [ "${LS_TARGET}" != "${FROOT}/opt/uutils/uutils" ]; then
  echo "[proof] Unexpected ls target: ${LS_TARGET}" >&2
  exit 1
fi
echo "${LS_VERSION}" | grep -iq "uutils" || { echo "[proof] ls --version does not mention uutils" >&2; exit 1; }

echo "[OK] uutils-coreutils is active."
SCRIPT
