#!/usr/bin/env bash
set -euxo pipefail

# Ensure PATH contains /usr/local/bin ahead of others for login shells too
printf 'export PATH="/usr/local/bin:$PATH"\n' > /etc/profile.d/99-oxidizr-arch.sh
chmod 0644 /etc/profile.d/99-oxidizr-arch.sh

# Stage workspace into /root/project/oxidizr-arch so builds are isolated from bind mount quirks
mkdir -p /root/project/oxidizr-arch
cp -a /workspace/. /root/project/oxidizr-arch/
cd /root/project/oxidizr-arch

# Prepare rust toolchain and ensure cargo is on PATH
if [ -f "/root/.cargo/env" ]; then
  # shellcheck disable=SC1091
  source "/root/.cargo/env"
fi
rustup default stable || true
if ! command -v cargo >/dev/null 2>&1; then
  echo "[prepare] cargo not found in PATH; sourcing /root/.cargo/env if present" >&2
  if [ -f "/root/.cargo/env" ]; then
    # shellcheck disable=SC1091
    source "/root/.cargo/env"
  fi
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo "[prepare] cargo still not found; installing stable toolchain explicitly" >&2
  rustup toolchain install stable || true
  # shellcheck disable=SC1091
  [ -f "/root/.cargo/env" ] && source "/root/.cargo/env"
fi
cargo --version || true
rustc --version || true

# Build product
cargo build --release

# Install convenient links
ln -sf "$PWD/target/release/oxidizr-arch" /usr/local/bin/oxidizr-arch
ln -sf "/usr/local/bin/oxidizr-arch" /usr/bin/oxidizr-arch || true

# Show what we installed
ls -l /usr/local/bin/oxidizr-arch || true
ls -l /usr/bin/oxidizr-arch || true
which oxidizr-arch || true
oxidizr-arch --help || true

# Drop into interactive login shell
exec bash -l
