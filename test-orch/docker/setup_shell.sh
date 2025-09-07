#!/bin/bash
set -e

echo "==> Building oxidizr-arch for interactive shell..."

# Navigate to the project directory inside the container
cd /workspace

# Build the release binary
cargo build --release

# Symlink the binary to a directory in the PATH
ln -sf "$(pwd)/target/release/oxidizr-arch" /usr/local/bin/oxidizr-arch

echo "==> Build complete. 'oxidizr-arch' is now in your PATH."
