#!/usr/bin/env bash
set -euo pipefail

if ! command -v python3 >/dev/null 2>&1; then
  pacman -Syy --noconfirm
  pacman -S --noconfirm --needed python python-yaml
fi

python3 /workspace/test-orch/container-runner-2/runner.py deps
