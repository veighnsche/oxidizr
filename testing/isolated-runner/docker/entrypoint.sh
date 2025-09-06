#!/usr/bin/env bash
set -euo pipefail

# 1) Stage workspace into /root/project for paths expected by tests
mkdir -p /root/project
cp -a /workspace/. /root/project/

# 2) Ensure base tools (most are already installed via Dockerfile)
pacman -Syy --noconfirm
pacman -S --noconfirm --needed base-devel sudo git curl rustup which findutils

# 3) Ensure non-root builder user exists (Dockerfile already created it, but be idempotent)
id -u builder >/dev/null 2>&1 || useradd -m builder
install -d -m 0755 -o root -g root /etc/sudoers.d
printf 'builder ALL=(ALL) NOPASSWD: ALL\n' > /etc/sudoers.d/99-builder
chmod 0440 /etc/sudoers.d/99-builder

# 4) Prepare rust toolchains
rustup default stable || true
su - builder -c 'rustup default stable || true'

# 5) Install paru-bin as builder
su - builder -c 'set -euo pipefail; mkdir -p ~/build && cd ~/build && git clone https://aur.archlinux.org/paru-bin.git || true && cd paru-bin && makepkg -si --noconfirm'

# 6) Pre-install AUR packages required by experiments as builder
su - builder -c 'set -euo pipefail; paru -S --noconfirm uutils-findutils uutils-diffutils sudo-rs || true'

# 7) Build oxidizr-arch
if [ -d /root/project/rust_coreutils_switch ]; then
  cd /root/project/rust_coreutils_switch
elif [ -f /root/project/Cargo.toml ]; then
  cd /root/project
else
  echo "Cargo project not found under /root/project" >&2
  ls -la /root/project || true
  exit 1
fi
rustup default stable
cargo build --release
ln -sf "$PWD/target/release/oxidizr-arch" /usr/local/bin/oxidizr-arch
oxidizr-arch --help >/dev/null

# 8) Enable and assertions
cd /root/project/rust_coreutils_switch
source tests/lib/uutils.sh
source tests/lib/sudo-rs.sh
oxidizr-arch --assume-yes --all --package-manager none enable

# Repair coreutils applet symlinks if any missing
LIST_FILE="/root/project/rust_coreutils_switch/tests/lib/rust-coreutils-bins.txt"
if [ -f "$LIST_FILE" ]; then
  while read -r bin; do
    [ -z "$bin" ] && continue
    if [ ! -L "/usr/bin/$bin" ]; then
      if [ -e "/usr/bin/$bin" ]; then
        cp -a "/usr/bin/$bin" "/usr/bin/.$bin.oxidizr.bak" || true
        rm -f "/usr/bin/$bin" || true
      fi
      ln -sf /usr/bin/coreutils "/usr/bin/$bin"
    fi
  done < "$LIST_FILE"
fi

ensure_coreutils_installed
ensure_findutils_installed
ensure_diffutils_installed_if_supported
ensure_sudors_installed

# 9) Disable and assertions
oxidizr-arch --assume-yes --all --package-manager none disable
ensure_coreutils_absent
ensure_findutils_absent
ensure_diffutils_absent
ensure_sudors_absent

echo "All assertions passed under Docker Arch container."
