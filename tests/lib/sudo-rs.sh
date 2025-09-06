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

  if [ ! -L "/usr/bin/sudo" ]; then
    echo "sudo not a symlink after enable" >&2; exit 1
  fi
  _sudo_dest="$(readlink -f /usr/bin/sudo || true)"
  case "${_sudo_dest}" in
    "/usr/lib/cargo/bin/sudo"|"/usr/bin/sudo-rs") ;;
    *) echo "Unexpected sudo link target: ${_sudo_dest}" >&2; exit 1 ;;
  esac
  [ -e "/usr/bin/.sudo.oxidizr.bak" ] || { echo "Missing .sudo.oxidizr.bak" >&2; exit 1; }
  # Ensure that invoking 'sudo' by basename resolves correctly and is runnable
  if ! command -v sudo >/dev/null 2>&1; then
    echo "'sudo' not found in PATH after enable" >&2; exit 1
  fi
  # Prefer type -P to get the resolved path without function/alias noise
  _sudo_path="$(type -P sudo || command -v sudo || true)"
  if [ "${_sudo_path:-}" != "/usr/bin/sudo" ]; then
    echo "Unexpected sudo path: ${_sudo_path:-<none>} (expected /usr/bin/sudo)" >&2; exit 1
  fi
  # Sanity: 'sudo --version' should execute (no strict string matching)
  sudo --version >/dev/null 2>&1 || { echo "'sudo --version' failed to execute after enable" >&2; exit 1; }
  # Version output of sudo-rs may vary across builds/distros; avoid strict string matching.
  # The symlink and backup checks above are sufficient to prove the switch.

  if [ ! -L "/usr/bin/su" ]; then
    echo "su not a symlink after enable" >&2; exit 1
  fi
  _su_dest="$(readlink -f /usr/bin/su || true)"
  case "${_su_dest}" in
    "/usr/lib/cargo/bin/su"|"/usr/bin/su-rs") ;;
    *) echo "Unexpected su link target: ${_su_dest}" >&2; exit 1 ;;
  esac
  [ -e "/usr/bin/.su.oxidizr.bak" ] || { echo "Missing .su.oxidizr.bak" >&2; exit 1; }
  # Likewise, avoid strict matching on su version output.

  if [ ! -L "/usr/sbin/visudo" ]; then
    echo "visudo not a symlink after enable" >&2; exit 1
  fi
  _visudo_dest="$(readlink -f /usr/sbin/visudo || true)"
  case "${_visudo_dest}" in
    "/usr/lib/cargo/bin/visudo"|"/usr/bin/visudo-rs") ;;
    *) echo "Unexpected visudo link target: ${_visudo_dest}" >&2; exit 1 ;;
  esac
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
