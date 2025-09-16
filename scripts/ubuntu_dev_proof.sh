#!/usr/bin/env bash
set -euo pipefail

# Non-interactive proof: disposable Ubuntu container, apply replacements
# (offline artifacts) and print version/proof that uutils is active.

IMG="ubuntu:24.04"
WORKDIR="/work"

docker run --rm -t -v "$(pwd)":${WORKDIR} -w ${WORKDIR} ${IMG} bash -s <<'SCRIPT'
set -euo pipefail
apt-get update
apt-get install -y curl ca-certificates build-essential pkg-config git jq xz-utils tar
curl https://sh.rustup.rs -sSf | sh -s -- -y
. "$HOME/.cargo/env"

# Build oxidizr-deb
cargo build -p oxidizr-deb
OXI="target/debug/oxidizr-deb"

# Prepare offline artifacts
mkdir -p /opt/uutils /opt/uutils-findutils

# Fetch uutils-coreutils release binary
URL=$(curl -sSL https://api.github.com/repos/uutils/coreutils/releases/latest | jq -r '.assets[] | select(.name|test("x86_64-unknown-linux-gnu.tar.xz$")) | .browser_download_url' | head -n1)
if [ -z "$URL" ]; then echo "[proof] Failed to locate uutils-coreutils release URL" >&2; exit 1; fi
TMPDIR=$(mktemp -d)
curl -L "$URL" -o "$TMPDIR/uutils.tar.xz"
tar -C "$TMPDIR" -xJf "$TMPDIR/uutils.tar.xz"
UU=$(find "$TMPDIR" -type f -name uutils -perm -u+x | head -n1 || true)
if [ -z "$UU" ]; then echo "[proof] uutils binary not found in archive" >&2; exit 1; fi
install -Dm0755 "$UU" "/opt/uutils/uutils"

# Provide a minimal uutils-findutils stub
echo -e "#!/usr/bin/env bash\necho uutils-findutils-dev-shell-stub" > /opt/uutils-findutils/uutils-findutils
chmod 0755 /opt/uutils-findutils/uutils-findutils

# Apply replacements using offline artifacts
"$OXI" --commit use coreutils --offline --use-local /opt/uutils/uutils
"$OXI" --commit use findutils --offline --use-local /opt/uutils-findutils/uutils-findutils

# Proof: show link target and version
set +e
LS_TARGET=$(readlink -f /usr/bin/ls || true)
LS_VERSION=$(ls --version 2>&1 | head -n2 || true)
set -e

echo "[PROOF] /usr/bin/ls -> ${LS_TARGET}"
echo "[PROOF] ls --version:\n${LS_VERSION}"

if [ "${LS_TARGET}" != "/opt/uutils/uutils" ]; then
  echo "[proof] Unexpected ls target: ${LS_TARGET}" >&2
  exit 1
fi
echo "${LS_VERSION}" | grep -iq "uutils" || { echo "[proof] ls --version does not mention uutils" >&2; exit 1; }

echo "[OK] uutils-coreutils is active."
SCRIPT
