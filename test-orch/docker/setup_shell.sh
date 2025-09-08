#!/bin/bash
set -euo pipefail

# Ensure a 'builder' user exists with passwordless sudo (needed by the demo's sudo checks)
if ! id -u builder >/dev/null 2>&1; then
  useradd -m builder
fi
echo 'builder ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/99-builder

echo "==> Building oxidizr-arch for interactive shell..."

# Navigate to the project directory inside the container
cd /workspace

# Ensure Rust toolchain is configured once (root only) with a minimal profile
rustup set profile minimal
if ! rustup toolchain list | grep -q '^stable'; then
  rustup default stable
fi

# Build the release binary
cargo build --release

# Symlink the binary to a directory in the PATH
ln -sf "$(pwd)/target/release/oxidizr-arch" /usr/local/bin/oxidizr-arch

echo "==> Build complete. 'oxidizr-arch' is now in your PATH."

# Interactive hint (printed after noisy build output so it's visible)
echo "Tip: Run the demo with: demo-utilities.sh --cleanup"
echo "Note: To compare behavior, manually run: 'oxidizr-arch enable --yes' or 'oxidizr-arch disable --yes --all' and then re-run the demo."
