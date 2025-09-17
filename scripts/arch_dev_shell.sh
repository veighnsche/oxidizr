#!/usr/bin/env bash
set -euo pipefail
set -x

# Interactive Arch container with oxidizr-arch installed from the current workspace.
# Nothing touches your host. All mutations (including /usr/bin) happen inside the container.
# Usage:
#   bash scripts/arch_dev_shell.sh
# Then inside the shell:
#   which oxidizr-arch && oxidizr-arch --help
#   which ls && ls --version | head -n2
#   oxidizr-arch doctor --json | jq .
#   oxidizr-arch status --json | jq .
#   # If implemented:
#   # oxidizr-arch --all --commit enable
#   # or granular: oxidizr-arch --experiments coreutils --commit enable
#   # After enabling, re-check a few applets:
#   # ls --version | head -n2
#   # printf --version | head -n2

IMG="archlinux:base-devel"
WORK="/work"

docker run --rm -it -v "$(pwd)":"${WORK}" -w "${WORK}" "$IMG" bash -lc '
set -euo pipefail
set -x

# Update and install base tooling
pacman -Syu --noconfirm
pacman -Sy --noconfirm archlinux-keyring || true
pacman -Syu --noconfirm
pacman -S --needed --noconfirm git sudo which jq tar xz curl rust cargo base-devel

# Create a build user for AUR helper if needed
id builder >/dev/null 2>&1 || useradd -m builder
echo "%wheel ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/wheel
usermod -aG wheel builder || true

# Build oxidizr-arch from mounted workspace
cargo build -p oxidizr-arch --release --locked
install -Dm0755 target/release/oxidizr-arch /usr/local/bin/oxidizr-arch
oxidizr-arch --help || true

# Testing policy: the product must perform all state mutations. The harness must not
# install or mutate product-managed artifacts. Only ensure infra (AUR helper) exists.
# The CLI is responsible for installing replacements during `use`/`replace`.

# Ensure an AUR helper (paru) is available for AUR-only packages
if ! command -v paru >/dev/null 2>&1; then
  sudo -u builder bash -lc "cd && (git clone https://aur.archlinux.org/paru-bin.git || true) && cd paru-bin && git pull --rebase || true && makepkg -si --noconfirm"
fi

# Drop to an interactive shell for manual testing
export OXI_AUR_HELPER_USER=builder
cat <<EOF

You are now in a safe Arch container shell. Suggestions:
  which oxidizr-arch && oxidizr-arch --help
  oxidizr-arch doctor --json | jq .
  oxidizr-arch status --json | jq .
  # Install replacements via CLI (not the harness):
  #   oxidizr-arch --commit use coreutils
  #   oxidizr-arch --commit use findutils
  #   oxidizr-arch --commit use sudo
  # Then re-check applets and versions inside this shell as needed.
EOF

exec bash -l
'
