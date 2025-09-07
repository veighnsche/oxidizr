#!/usr/bin/env bash
set -euo pipefail

# Ensure a sane PATH that prefers /usr/bin for coreutils
export PATH="/usr/bin:/usr/sbin:/bin:/sbin:${PATH:-}"
export RUST_LOG="info"

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

# Optional: run YAML suites (Spread-style execute blocks) inside Docker when requested
# Trigger via environment variable RUN_SPREAD_SUITES=1 or by creating a sentinel file
# either at /workspace/.run_spread_suites or in the project root.
if [ "${RUN_SPREAD_SUITES:-0}" = "1" ] || \
   [ -f "/workspace/.run_spread_suites" ] || \
   [ -f "/root/project/oxidizr-arch/.run_spread_suites" ]; then
  echo "[entrypoint] RUN_SPREAD_SUITES requested; executing YAML suites..."
  bash "/root/project/oxidizr-arch/test-orch/docker/run_yaml_suites.sh"
  echo "[entrypoint] YAML suites finished. Exiting entrypoint."
  exit 0
fi

# Show GNU state before enabling uutils
show_coreutils_snapshot pre-enable

# Concise demo sequence requested: ls --version, enable, ls --version
# Print the first line only to keep it tidy.
ls --version 2>&1 | head -n 1
echo "enable"

oxidizr-arch --assume-yes --experiments coreutils,sudo-rs --package-manager none enable

# After enabling, show ls version again (first line)
ls --version 2>&1 | head -n 1

# Show uutils state after enabling
show_coreutils_snapshot post-enable

# Do NOT add masking workarounds (e.g., 'hash -r') here. If applet resolution fails
# after enable, fix the product or run assertions in a fresh process. The harness must
# not hide product failures with shell-level cache flushes.

# Ensure required toolsets are installed after enabling (quiet with summary)
echo "[assert] coreutils installed: running..."
ensure_coreutils_installed
echo "[assert] diffutils installed (if supported): running..."
ensure_diffutils_installed_if_supported
echo "[assert] sudo-rs installed: running..."
ensure_sudors_installed

# 9) Disable and assertions
echo "disable"
oxidizr-arch --assume-yes --experiments coreutils,sudo-rs --package-manager none disable
# After disabling, show ls version once more (first line)
ls --version 2>&1 | head -n 1
show_coreutils_snapshot post-disable
echo "[assert] coreutils absent: running..."
ensure_coreutils_absent
echo "[assert] diffutils absent: running..."
ensure_diffutils_absent
echo "[assert] sudo-rs absent: running..."
ensure_sudors_absent

echo "All assertions passed under Docker Arch container."
