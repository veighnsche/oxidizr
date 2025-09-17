#!/usr/bin/env bash
set -euo pipefail
set -x

# Non-interactive proof: disposable Arch container, build and install oxidizr-arch
# from the workspace, attempt to install replacements, then try to enable them via
# the CLI and print diagnostic output. Host system is not modified.

IMG="archlinux:base-devel"
WORKDIR="/work"

docker run --rm -i -v "$(pwd)":"${WORKDIR}" -w "${WORKDIR}" "$IMG" bash -s <<'SCRIPT'
set -euo pipefail
set -x

# Refresh packages and install prerequisites
pacman -Syu --noconfirm
pacman -S --needed --noconfirm git sudo which jq tar xz curl rust cargo base-devel python

# Create a build user for AUR helper install
id builder >/dev/null 2>&1 || useradd -m builder
echo "%wheel ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/wheel
usermod -aG wheel builder || true

# Build oxidizr-arch
cargo build -p oxidizr-arch --release --locked
OXI="target/release/oxidizr-arch"
install -Dm0755 "$OXI" /usr/local/bin/oxidizr-arch

# Pre-state diagnostics
set +e
which oxidizr-arch || true
oxidizr-arch --version || true
which ls || true
LS_PRE=$(ls --version 2>&1 | head -n2 || true)
set -e

echo "[PROOF] pre ls --version:\n${LS_PRE}"

# Testing policy: the product must perform all state mutations. The harness must not
# install or mutate product-managed artifacts. We only ensure infra (AUR helper) exists.
# The CLI is responsible for installing replacements during `use`/`replace`.

# Ensure an AUR helper (paru) is available for packages that require AUR
if ! command -v paru >/dev/null 2>&1; then
  sudo -u builder bash -lc "cd && (git clone https://aur.archlinux.org/paru-bin.git || true) && cd paru-bin && git pull --rebase || true && makepkg -si --noconfirm" || true
fi

# CLI diagnostics
oxidizr-arch doctor --json | tee /tmp/arch_doctor.json || true
oxidizr-arch status --json | tee /tmp/arch_status.json || true

# Attempt to enable replacements via oxidizr-arch
# Prefer the "enable" subcommand if present; otherwise try "use" semantics.
if oxidizr-arch --help 2>&1 | grep -q "enable"; then
  oxidizr-arch --all --commit enable || true
elif oxidizr-arch --help 2>&1 | grep -q "use "; then
  # Fall back to package-by-package enable semantics if supported
  oxidizr-arch --commit use coreutils || true
  oxidizr-arch --commit use findutils || true
  oxidizr-arch --commit use sudo || true
else
  echo "[info] Neither 'enable' nor 'use' detected in CLI help; skipping enable step"
fi

# Status after apply (ground truth from CLI)
echo "[PROOF] status after use:" 
oxidizr-arch status --json > /tmp/arch_status_after.json || true
python - <<'PY'
import json, sys
try:
  data=json.load(open('/tmp/arch_status_after.json'))
  print('[PROOF] coreutils status:', data.get('coreutils'))
  print('[PROOF] findutils status:', data.get('findutils'))
  print('[PROOF] sudo status:', data.get('sudo'))
except Exception as e:
  print('[PROOF] status parse error:', e)
PY

# Post-state diagnostics (tool-agnostic)
echo "[PROOF] which oxidizr-arch: $(command -v oxidizr-arch || true)"

LS_PATH="/usr/bin/ls"
UU_PATH="/usr/bin/uutils"
echo "[SH] islink(ls): $( [ -L "$LS_PATH" ] && echo true || echo false )"
TARGET=""
if [ -L "$LS_PATH" ]; then
  if command -v readlink >/dev/null 2>&1; then
    TARGET=$(readlink "$LS_PATH" || true)
  elif command -v python >/dev/null 2>&1; then
    TARGET=$(python - <<'PY'
import os
print(os.readlink("/usr/bin/ls"))
PY
)
  fi
  case "$TARGET" in $'\n'*) TARGET=${TARGET#$'\n'};; esac
  case "$TARGET" in *$'\n') TARGET=${TARGET%$'\n'};; esac
fi
echo "[SH] link_target(ls): ${TARGET}"
echo "[SH] exists(target): $( [ -n "$TARGET" ] && [ -e "$TARGET" ] && echo true || echo false )"
echo "[SH] exec(target): $( [ -n "$TARGET" ] && [ -x "$TARGET" ] && echo true || echo false )"

if [ -L "$LS_PATH" ] && [ -n "$TARGET" ] && [ -x "$TARGET" ]; then
  echo "[PROOF] SUCCESS: /usr/bin/ls is a symlink to an executable replacement (${TARGET})."
else
  echo "[PROOF] WARN: Replacement not verified (either no symlink or target not executable)."
fi
SCRIPT
