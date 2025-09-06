#!/usr/bin/env bash
set -euo pipefail

# Ensure a predictable PATH
export PATH="/usr/local/bin:/usr/bin:/usr/sbin:/bin:/sbin:${PATH:-}"

# 1) Stage workspace into /root/project/oxidizr-arch (idempotent)
mkdir -p /root/project/oxidizr-arch
if [ ! -e /root/project/oxidizr-arch/Cargo.toml ]; then
  cp -a /workspace/. /root/project/oxidizr-arch/
fi
cd /root/project/oxidizr-arch

# 2) Build product and expose it
if ! command -v cargo >/dev/null 2>&1; then
  if [ -f "/root/.cargo/env" ]; then
    # shellcheck disable=SC1091
    source "/root/.cargo/env"
  fi
fi
rustup default stable || true
cargo build --release
ln -sf "$PWD/target/release/oxidizr-arch" /usr/local/bin/oxidizr-arch
oxidizr-arch --help >/dev/null

# 3) Source helpers
source tests/lib/uutils.sh
source tests/lib/sudo-rs.sh

# 4) Enable sudo-rs (together with coreutils for parity with entrypoint)
oxidizr-arch --assume-yes --experiments coreutils,sudo-rs --package-manager none enable

# 5) Assert installed and switched
ensure_sudors_installed

# 6) Disable and assert absent
oxidizr-arch --assume-yes --experiments coreutils,sudo-rs --package-manager none disable
ensure_sudors_absent

echo "[sudo-tests] OK: enable/disable assertions passed"
