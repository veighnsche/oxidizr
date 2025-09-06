#!/usr/bin/env bash
set -euo pipefail

# Ensure a sane PATH that prefers /usr/bin for coreutils
export PATH="/usr/bin:/usr/sbin:/bin:/sbin:${PATH:-}"
export RUST_LOG="info"

# 1) Stage workspace into /root/project/oxidizr-arch for paths expected by tests
mkdir -p /root/project/oxidizr-arch
cp -a /workspace/. /root/project/oxidizr-arch/

# 2) Ensure base tools (most are already installed via Dockerfile)
pacman -Syy --noconfirm
pacman -S --noconfirm --needed base-devel sudo git curl rustup which findutils

# 3) Ensure non-root builder user exists (Dockerfile already created it, but be idempotent)
id -u builder >/dev/null 2>&1 || useradd -m builder
install -d -m 0755 -o root -g root /etc/sudoers.d
printf 'builder ALL=(ALL) NOPASSWD: ALL\n' > /etc/sudoers.d/99-builder
chmod 0440 /etc/sudoers.d/99-builder

# 4) Prepare rust toolchains (for building the project only)
rustup default stable || true
su - builder -c 'rustup default stable || true'

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
rustup default stable
cargo build --release
ln -sf "$PWD/target/release/oxidizr-arch" /usr/local/bin/oxidizr-arch
oxidizr-arch --help >/dev/null

# 8) Enable and assertions
cd /root/project/oxidizr-arch
source tests/lib/uutils.sh
source tests/lib/sudo-rs.sh
# Pre-enable: capture a few applet resolutions in this fresh shell
echo "[pre-enable] command -V readlink"; command -V readlink || true
echo "[pre-enable] ls -l /usr/bin/readlink"; ls -l /usr/bin/readlink || true
echo "[pre-enable] readlink --version (via /usr/bin if needed)"; /usr/bin/readlink --version || /usr/bin/coreutils --coreutils-prog=readlink --version || true
echo "[pre-enable] ls -l /usr/bin/{cp,ln,rm}"; ls -l /usr/bin/{cp,ln,rm} || true

oxidizr-arch --assume-yes --experiments coreutils,sudo-rs --package-manager none enable

# Post-enable: show exact symlink destinations for a sample of applets
echo "[post-enable] command -V readlink"; command -V readlink || true
echo "[post-enable] ls -l /usr/bin/readlink"; ls -l /usr/bin/readlink || true
echo "[post-enable] /usr/bin/readlink --version (or via coreutils dispatch)"; /usr/bin/readlink --version || /usr/bin/coreutils --coreutils-prog=readlink --version || true
echo "[post-enable] ls -l /usr/bin/{cp,ln,rm}"; ls -l /usr/bin/{cp,ln,rm} || true

# Do NOT add masking workarounds (e.g., 'hash -r') here. If applet resolution fails
# after enable, fix the product or run assertions in a fresh process. The harness must
# not hide product failures with shell-level cache flushes.

# Ensure required toolsets are installed after enabling
ensure_coreutils_installed
ensure_diffutils_installed_if_supported
ensure_sudors_installed

# 9) Disable and assertions
oxidizr-arch --assume-yes --experiments coreutils,sudo-rs --package-manager none disable
ensure_coreutils_absent
ensure_diffutils_absent
ensure_sudors_absent

echo "All assertions passed under Docker Arch container."
