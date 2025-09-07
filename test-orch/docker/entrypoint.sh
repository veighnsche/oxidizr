#!/usr/bin/env bash
set -euo pipefail

# Ensure a sane PATH that prefers /usr/bin for coreutils
export PATH="/usr/bin:/usr/sbin:/bin:/sbin:${PATH:-}"

# Helper: show how selected coreutils resolve and which implementation they are
show_coreutils_snapshot() {
  local label="$1"; shift || true
  local apps=(readlink ls cp mv rm ln mkdir rmdir touch date echo)
  echo "[snapshot:$label] ==== Applet resolution & versions ===="
  for a in "${apps[@]}"; do
    echo "[snapshot:$label] -- $a --"
    command -V "$a" || true
    if [ -e "/usr/bin/$a" ]; then
      ls -l "/usr/bin/$a" || true
      dest=$(readlink -f "/usr/bin/$a" || true)
      echo "[snapshot:$label] /usr/bin/$a -> ${dest}" || true
    fi
    # Run via shell-resolved path to surface hashing/PATH issues
    "$a" --version 2>&1 | head -n 1
  done
  echo "[snapshot:$label] ==================================="
}

# Verbosity: VERBOSE=0 (quiet), 1 (normal), 2 (verbose)
: "${VERBOSE:=1}"
logv() { [ "${VERBOSE}" -ge 1 ] && echo "$*" || true; }
logvv() { [ "${VERBOSE}" -ge 2 ] && echo "$*" || true; }

# Suppress command output at normal verbosity; show only at VERBOSE>=2
runq() {
  if [ "${VERBOSE}" -ge 2 ]; then
    "$@"
  else
    "$@" >/dev/null 2>&1 || return $?
  fi
}

# Map VERBOSE to RUST_LOG unless user explicitly provided RUST_LOG
if [ -z "${RUST_LOG:-}" ]; then
  case "${VERBOSE}" in
    0) export RUST_LOG="error" ;;
    1) export RUST_LOG="error" ;;
    2) export RUST_LOG="warn" ;;
    *) export RUST_LOG="info" ;;
  esac
fi

# 1) Stage workspace into /root/project/oxidizr-arch for paths expected by tests
mkdir -p /root/project/oxidizr-arch
cp -a /workspace/. /root/project/oxidizr-arch/

# 2) Ensure base tools (most are already installed via Dockerfile)
runq pacman -Syy --noconfirm
runq pacman -S --noconfirm --needed base-devel sudo git curl rustup which findutils

# 3) Ensure non-root builder user exists (Dockerfile already created it, but be idempotent)
id -u builder >/dev/null 2>&1 || useradd -m builder
install -d -m 0755 -o root -g root /etc/sudoers.d
printf 'builder ALL=(ALL) NOPASSWD: ALL\n' > /etc/sudoers.d/99-builder
chmod 0440 /etc/sudoers.d/99-builder

# 3b) Install AUR helper (paru-bin) as builder for suites requiring AUR packages
if ! command -v paru >/dev/null 2>&1; then
  logv "[prepare] Installing paru-bin (AUR helper) for test suites"
  su - builder -c 'mkdir -p ~/build && cd ~/build && git clone https://aur.archlinux.org/paru-bin.git || true && cd paru-bin && makepkg -si --noconfirm'
fi

# 4) Prepare rust toolchains (for building the project only)
runq rustup default stable || true
runq su - builder -c 'rustup default stable || true'

# Note: Do not pre-mutate /usr/bin applets here. The product (oxidizr-arch)
# performs safe, syscall-based switching during 'enable'.

# 7) Build oxidizr-arch (assume /root/project/oxidizr-arch is the repository root)
if [ -f /root/project/oxidizr-arch/Cargo.toml ]; then
  cd /root/project/oxidizr-arch
else
  echo "Cargo project not found under /root/project/oxidizr-arch" >&2
  ls -la /root/project/oxidizr-arch || true
  exit 1
fi
rustup default stable >/dev/null 2>&1 || true
: "${CARGO_BUILD_JOBS:=2}"
logv "[build] cargo build --release -j ${CARGO_BUILD_JOBS}"
runq cargo build --release -j "${CARGO_BUILD_JOBS}"
runq ln -sf "$PWD/target/release/oxidizr-arch" /usr/local/bin/oxidizr-arch
runq oxidizr-arch --help

# 8) Enable and assertions
cd /root/project/oxidizr-arch
source tests/lib/uutils.sh
source tests/lib/sudo-rs.sh

# REQUIRED: Run YAML suites (Spread-style execute blocks) inside Docker first.
logv "[entrypoint] Running YAML suites (required)"
bash "/root/project/oxidizr-arch/test-orch/docker/run_yaml_suites.sh"
logv "[entrypoint] YAML suites finished. Proceeding to demo/assertion flow."

# Show GNU state before enabling uutils (verbose only)
[ "${VERBOSE}" -ge 2 ] && show_coreutils_snapshot pre-enable || true

# Concise demo sequence requested: ls --version, enable, ls --version (verbose only)
[ "${VERBOSE}" -ge 2 ] && ls --version 2>&1 | head -n 1 || true
logv "enable"

oxidizr-arch --assume-yes --experiments coreutils,sudo-rs --package-manager none enable

# After enabling, show ls version again (first line, verbose only)
[ "${VERBOSE}" -ge 2 ] && ls --version 2>&1 | head -n 1 || true

# Show uutils state after enabling (verbose only)
[ "${VERBOSE}" -ge 2 ] && show_coreutils_snapshot post-enable || true

# Do NOT add masking workarounds (e.g., 'hash -r') here. If applet resolution fails
# after enable, fix the product or run assertions in a fresh process. The harness must
# not hide product failures with shell-level cache flushes.

# Ensure required toolsets are installed after enabling (quiet with summary)
logv "[assert] coreutils installed: running..."
ensure_coreutils_installed
logv "[assert] diffutils installed (if supported): running..."
ensure_diffutils_installed_if_supported
logv "[assert] sudo-rs installed: running..."
ensure_sudors_installed

# 9) Disable and assertions
logv "disable"
oxidizr-arch --assume-yes --experiments coreutils,sudo-rs --package-manager none disable
[ "${VERBOSE}" -ge 2 ] && ls --version 2>&1 | head -n 1 || true
[ "${VERBOSE}" -ge 2 ] && show_coreutils_snapshot post-disable || true
logv "[assert] coreutils absent: running..."
ensure_coreutils_absent
logv "[assert] diffutils absent: running..."
ensure_diffutils_absent
logv "[assert] sudo-rs absent: running..."
ensure_sudors_absent

logv "All assertions passed under Docker Arch container."
