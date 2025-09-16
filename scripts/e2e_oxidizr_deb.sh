#!/usr/bin/env bash
set -euo pipefail

# Local E2E for oxidizr-deb on Ubuntu/Debian
# - builds oxidizr-deb
# - prepares hermetic root
# - fetches uutils-coreutils multi-call binary
# - commits use coreutils offline into root
# - verifies ls --version includes 'uutils'
# - runs status --json and doctor --json smoke checks

ROOT=""
WORKDIR=""
CLEANUP() {
  local code=$?
  if [[ -n "${WORKDIR}" && -d "${WORKDIR}" ]]; then rm -rf "${WORKDIR}" || true; fi
  # keep ROOT for inspection
  exit $code
}
trap CLEANUP EXIT

# Check deps
need_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "Missing dependency: $1" >&2; exit 1; }
}
need_cmd cargo
need_cmd curl
need_cmd jq
need_cmd tar
need_cmd xz

# Build CLI
cargo build -p oxidizr-deb --release

# Prepare hermetic root
ROOT=$(mktemp -d)
mkdir -p "$ROOT/usr/bin" "$ROOT/var/lock" "$ROOT/opt/uutils"

echo "[e2e] root=$ROOT"

# Determine target triple asset suffix by arch
ARCH=$(uname -m)
TRIPLE=""
case "$ARCH" in
  x86_64) TRIPLE="x86_64-unknown-linux-gnu" ;;
  aarch64) TRIPLE="aarch64-unknown-linux-gnu" ;;
  *) echo "Unsupported arch: $ARCH" >&2; exit 1 ;;
esac

fetch_and_install_uutils() {
  local workdir=$(mktemp -d)
  # Try to find a suitable asset with either xz or gz compression
  local url=$(curl -sSL https://api.github.com/repos/uutils/coreutils/releases/latest \
    | jq -r --arg arch "$ARCH" '.assets[] | select((.name|test($arch)) and (.name|test("linux")) and (.name|test("\\.tar\\.(xz|gz)$"))) | .browser_download_url' \
    | head -n1)

  if [[ -z "$url" ]]; then
    echo "[e2e] no prebuilt uutils asset found for arch=$ARCH; falling back to building from source" >&2
    build_uutils_from_source || return 1
    return 0
  fi

  echo "[e2e] downloading: $url"
  local tarball="$workdir/uutils.tar"
  curl -L "$url" -o "$tarball"

  echo "[e2e] extracting"
  if [[ "$url" =~ \.tar\.xz$ ]]; then
    tar -C "$workdir" -xJf "$tarball"
  else
    tar -C "$workdir" -xzf "$tarball"
  fi
  local uu=$(find "$workdir" -type f -name uutils -perm -u+x | head -n1 || true)
  if [[ -z "$uu" ]]; then
    echo "[e2e] uutils binary not found in release archive; building from source" >&2
    build_uutils_from_source || return 1
  else
    install -Dm0755 "$uu" "$ROOT/opt/uutils/uutils"
  fi
}

build_uutils_from_source() {
  echo "[e2e] cloning uutils/coreutils and building release uutils binary" >&2
  local src=$(mktemp -d)
  git clone --depth 1 https://github.com/uutils/coreutils.git "$src"
  (cd "$src" && cargo build --release -p coreutils)
  # Multi-call binary is named 'uutils' in target dir
  local uu=$(find "$src/target/release" -maxdepth 1 -type f -name uutils -perm -u+x | head -n1 || true)
  if [[ -z "$uu" ]]; then
    echo "[e2e] failed to build uutils binary from source" >&2
    return 1
  fi
  install -Dm0755 "$uu" "$ROOT/opt/uutils/uutils"
}

fetch_and_install_uutils

# Switch coreutils to uutils in the hermetic root
./target/release/oxidizr-deb --root "$ROOT" --commit --assume-yes use coreutils --offline --use-local "$ROOT/opt/uutils/uutils"

test -L "$ROOT/usr/bin/ls"

# Verify ls is from uutils
set +e
"$ROOT/usr/bin/ls" --version > /tmp/ls_version.txt 2>&1
set -e
cat /tmp/ls_version.txt

grep -i "uutils" /tmp/ls_version.txt

# Status JSON smoke check
./target/release/oxidizr-deb --root "$ROOT" status --json | tee /tmp/status.json
[[ "$(jq -r .coreutils /tmp/status.json)" == "active" ]]

# Doctor JSON smoke check
./target/release/oxidizr-deb --root "$ROOT" doctor --json | tee /tmp/doctor.json
[[ "$(jq -r .paths_ok /tmp/doctor.json)" == "true" ]]
[[ "$(jq -r .locks_present /tmp/doctor.json)" == "false" ]]

echo "[e2e] SUCCESS: oxidizr-deb use coreutils worked; ls is from uutils"
