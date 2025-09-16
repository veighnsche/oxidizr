#!/usr/bin/env bash
set -euo pipefail

# Full destructive smoke inside a disposable Ubuntu container.
# This script should be run on a host with Docker installed.
# It will:
#  - start an ubuntu:24.04 container
#  - install rust toolchain and build oxidizr-deb
#  - run SAFE BDD tests (fakeroot + dry-run)
#  - run a DESTRUCTIVE smoke within the container's live /:
#       * --commit use coreutils
#       * --commit replace coreutils  (purges GNU coreutils under guardrails)
#       * --commit restore coreutils  (reinstalls GNU coreutils and restores)
#  - exit the container (no host impact)

IMG="ubuntu:24.04"
WORKDIR="/work"

exec docker run --rm -t \
  -v "$(pwd)":${WORKDIR} \
  -w ${WORKDIR} \
  ${IMG} bash -lc '
set -euo pipefail
apt-get update
apt-get install -y curl ca-certificates build-essential pkg-config git
curl https://sh.rustup.rs -sSf | sh -s -- -y
. $HOME/.cargo/env

# Build and safe tests
cargo check -p oxidizr-deb
cargo test -p oxidizr-deb --features bdd -q

# Destructive smoke: do not run other commands between replace and restore
# Use under live root intentionally inside the container
oxidizr_deb_bin="target/debug/oxidizr-deb"

# 1) Commit use coreutils (ensures replacement install if missing and links)
"$oxidizr_deb_bin" --commit use coreutils || exit 1

# 2) Commit replace coreutils (purges GNU coreutils under guardrails)
#    After this point, coreutils applets (ls, cat, etc.) may be missing.
#    We rely only on bash builtins and apt-get for the next step.
"$oxidizr_deb_bin" --commit replace coreutils || exit 1

# 3) Commit restore coreutils (reinstall GNU coreutils and restore topology)
"$oxidizr_deb_bin" --commit restore coreutils || exit 1

# Quick sanity: ensure ls exists again (use explicit path to avoid aliases)
command -v ls >/dev/null 2>&1 || { echo "ls not found after restore" >&2; exit 1; }
ls --version >/dev/null 2>&1 || { echo "ls not functional after restore" >&2; exit 1; }
'
