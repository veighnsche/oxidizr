#!/usr/bin/env bash
set -euo pipefail

# share the MATCH/NOMATCH functions from uutils.sh when sourced after it

pkg_installed() {
  local pkg="$1"
  if command -v paru >/dev/null 2>&1; then
    paru -Qi "$pkg" >/dev/null 2>&1 && return 0
  fi
  if command -v yay >/dev/null 2>&1; then
    yay -Qi "$pkg" >/dev/null 2>&1 && return 0
  fi
  pacman -Qi "$pkg" >/dev/null 2>&1 && return 0
  return 1
}

ensure_sudors_installed() {
  pkg_installed sudo-rs || { echo "sudo-rs not installed" >&2; exit 1; }

  [ -L "/usr/bin/sudo" ] && [ "$(readlink -f /usr/bin/sudo)" = "/usr/lib/cargo/bin/sudo" ] || { echo "sudo not linked to sudo-rs" >&2; exit 1; }
  [ -e "/usr/bin/.sudo.oxidizr.bak" ] || { echo "Missing .sudo.oxidizr.bak" >&2; exit 1; }
  sudo --version 2>&1 | MATCH 'sudo-rs'

  [ -L "/usr/bin/su" ] && [ "$(readlink -f /usr/bin/su)" = "/usr/lib/cargo/bin/su" ] || { echo "su not linked to su-rs" >&2; exit 1; }
  [ -e "/usr/bin/.su.oxidizr.bak" ] || { echo "Missing .su.oxidizr.bak" >&2; exit 1; }
  su --version 2>&1 | MATCH 'su-rs'

  [ -L "/usr/sbin/visudo" ] && [ "$(readlink -f /usr/sbin/visudo)" = "/usr/lib/cargo/bin/visudo" ] || { echo "visudo not linked" >&2; exit 1; }
  [ -e "/usr/sbin/.visudo.oxidizr.bak" ] || { echo "Missing .visudo.oxidizr.bak" >&2; exit 1; }
}

ensure_sudors_absent() {
  if pkg_installed sudo-rs; then
    echo "sudo-rs unexpectedly installed" >&2; exit 1
  fi

  [ ! -L "/usr/bin/sudo" ] || { echo "Unexpected sudo symlink" >&2; exit 1; }
  [ ! -e "/usr/bin/.sudo.oxidizr.bak" ] || { echo "Unexpected .sudo.oxidizr.bak" >&2; exit 1; }
  sudo --version 2>&1 | NOMATCH 'sudo-rs'

  [ ! -L "/usr/bin/su" ] || { echo "Unexpected su symlink" >&2; exit 1; }
  [ ! -e "/usr/bin/.su.oxidizr.bak" ] || { echo "Unexpected .su.oxidizr.bak" >&2; exit 1; }
  su --version 2>&1 | NOMATCH 'su-rs'

  [ ! -L "/usr/sbin/visudo" ] || { echo "Unexpected visudo symlink" >&2; exit 1; }
  [ ! -e "/usr/sbin/.visudo.oxidizr.bak" ] || { echo "Unexpected .visudo.oxidizr.bak" >&2; exit 1; }
}
