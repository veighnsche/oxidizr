#!/usr/bin/env bash
set -euo pipefail

# Resolve realpath robustly even if the readlink applet symlink is not yet present
_rl() {
  if command -v readlink >/dev/null 2>&1; then
    readlink "$@"
  elif [ -x /usr/bin/coreutils ]; then
    /usr/bin/coreutils --coreutils-prog=readlink "$@"
  else
    echo "readlink unavailable (no applet and no /usr/bin/coreutils)" >&2
    return 127
  fi
}

# Basic assertion helpers (compatible with pipe usage: cmd | MATCH 'text')
MATCH() {
  local pattern="$1"; shift || true
  local input
  input="$(cat)"
  if echo "$input" | grep -E -q "$pattern"; then
    echo "$input"
    exit 0
  else
    echo "Assertion failed: expected to match pattern: $pattern" >&2
    echo "$input" >&2
    exit 1
  fi
}

NOMATCH() {
  local pattern="$1"; shift || true
  local input
  input="$(cat)"
  if echo "$input" | grep -E -q "$pattern"; then
    echo "Assertion failed: expected not to match pattern: $pattern" >&2
    echo "$input" >&2
    exit 1
  else
    echo "$input"
    exit 0
  fi
}

# Resolve repository root and fixtures
_uutils_script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
_REPO_ROOT="${SPREAD_PATH:-$(cd "$_uutils_script_dir/../.." && pwd)}"
_BINS_LIST="${_REPO_ROOT}/tests/lib/rust-coreutils-bins.txt"

# Package query helpers (Arch)
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

_note_if_missing_backup() {
  local file="$1"
  if [ ! -e "$(dirname "$file")/.""$(basename "$file")"".oxidizr.bak" ]; then
    echo "Note: backup for $(basename "$file") not found (tolerated)" >&2
  fi
}

ensure_coreutils_installed() {
  pkg_installed uutils-coreutils || { echo "uutils-coreutils not installed" >&2; exit 1; }
  while IFS= read -r bin; do
    [ -z "$bin" ] && continue
    local target="/usr/bin/$bin"
    if [ ! -L "$target" ]; then
      echo "Expected symlink for $target" >&2; exit 1
    fi
    local dest
    dest="$(_rl -f "$target")"
    if [ "$bin" != "coreutils" ] && [ "$dest" != "/usr/bin/coreutils" ]; then
      echo "$target -> $dest" | MATCH '/usr/bin/coreutils' >/dev/null
    fi
    _note_if_missing_backup "$target"
    $bin --help 2>&1 | NOMATCH 'www.gnu.org/software/coreutils'
  done < "$_BINS_LIST"
}

ensure_coreutils_absent() {
  if pkg_installed uutils-coreutils; then
    echo "uutils-coreutils unexpectedly installed" >&2; exit 1
  fi
  local target="/usr/bin/date"
  if [ -L "$target" ]; then
    echo "Expected no symlink for $target" >&2; exit 1
  fi
  [ ! -e "/usr/bin/.date.oxidizr.bak" ] || { echo "Unexpected backup for date" >&2; exit 1; }
  date --help 2>&1 | MATCH 'GNU'
}

ensure_findutils_installed() {
  pkg_installed uutils-findutils || { echo "uutils-findutils not installed" >&2; exit 1; }
  [ -L "/usr/bin/find" ] && [ "$(readlink -f /usr/bin/find)" = "/usr/lib/cargo/bin/findutils/find" ] || { echo "find not linked correctly" >&2; exit 1; }
  [ -e "/usr/bin/.find.oxidizr.bak" ] || { echo "Missing .find.oxidizr.bak" >&2; exit 1; }
  find --help 2>&1 | NOMATCH 'www.gnu.org/software/findutils'

  [ -L "/usr/bin/xargs" ] && [ "$(readlink -f /usr/bin/xargs)" = "/usr/lib/cargo/bin/findutils/xargs" ] || { echo "xargs not linked correctly" >&2; exit 1; }
  [ -e "/usr/bin/.xargs.oxidizr.bak" ] || { echo "Missing .xargs.oxidizr.bak" >&2; exit 1; }
  xargs --help 2>&1 | NOMATCH 'www.gnu.org/software/findutils'
}

ensure_findutils_absent() {
  if pkg_installed uutils-findutils; then
    echo "uutils-findutils unexpectedly installed" >&2; exit 1
  fi
  [ ! -L "/usr/bin/find" ] || { echo "Unexpected symlink for /usr/bin/find" >&2; exit 1; }
  [ ! -e "/usr/bin/.find.oxidizr.bak" ] || { echo "Unexpected .find.oxidizr.bak" >&2; exit 1; }
  find --help 2>&1 | MATCH 'GNU'

  [ ! -L "/usr/bin/xargs" ] || { echo "Unexpected symlink for /usr/bin/xargs" >&2; exit 1; }
  [ ! -e "/usr/bin/.xargs.oxidizr.bak" ] || { echo "Unexpected .xargs.oxidizr.bak" >&2; exit 1; }
  xargs --help 2>&1 | MATCH 'GNU'
}

ensure_diffutils_installed_if_supported() {
  if pkg_installed uutils-diffutils; then
    [ -L "/usr/bin/diff" ] && [ "$(readlink -f /usr/bin/diff)" = "/usr/lib/cargo/bin/diffutils/diff" ] || { echo "diff not linked correctly" >&2; exit 1; }
    [ -e "/usr/bin/.diff.oxidizr.bak" ] || { echo "Missing .diff.oxidizr.bak" >&2; exit 1; }
    diff --help 2>&1 | NOMATCH 'www.gnu.org/software/diffutils'
  else
    echo "Skipping diffutils checks (package not installed/supported)"
  fi
}

ensure_diffutils_absent() {
  if pkg_installed uutils-diffutils; then
    echo "uutils-diffutils unexpectedly installed" >&2; exit 1
  fi
  # Reflects current file’s assertions referencing find
  [ ! -L "/usr/bin/find" ] || { echo "Unexpected symlink for /usr/bin/find" >&2; exit 1; }
  [ ! -e "/usr/bin/.find.oxidizr.bak" ] || { echo "Unexpected .find.oxidizr.bak (diffutils)" >&2; exit 1; }
  find --help 2>&1 | MATCH 'GNU'
}
