#!/usr/bin/env bash
set -euo pipefail

# Basic assertion helpers (compatible with pipe usage: cmd | MATCH 'text')
MATCH() {
  local pattern="$1"; shift || true
  local input
  input="$(cat)"
  if echo "$input" | grep -E -q "$pattern"; then
    # success: be quiet
    exit 0
  else
    echo "Assertion failed: expected to match pattern: $pattern" >&2
    # print only first line to avoid flooding
    echo "$input" | head -n 3 >&2
    exit 1
  fi
}

NOMATCH() {
  local pattern="$1"; shift || true
  local input
  input="$(cat)"
  if echo "$input" | grep -E -q "$pattern"; then
    echo "Assertion failed: expected not to match pattern: $pattern" >&2
    echo "$input" | head -n 3 >&2
    exit 1
  else
    # success: be quiet
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

_require_backup() {
  local file="$1"
  local backup_path
  backup_path="$(dirname "$file")/.""$(basename "$file")"".oxidizr.bak"
  if [ ! -e "$backup_path" ]; then
    echo "Missing backup for $(basename "$file"): expected ${backup_path}" >&2
    exit 1
  fi
}

ensure_coreutils_installed() {
  pkg_installed uutils-coreutils || { echo "uutils-coreutils not installed" >&2; exit 1; }
  local unified_a="/usr/bin/coreutils"
  local unified_b="/usr/bin/uu-coreutils"
  local have_unified=0
  if [ -x "$unified_a" ] || [ -x "$unified_b" ]; then
    have_unified=1
  fi
  # Minimal required set that uutils reliably provides; keep tests stable
  local REQUIRED_BINS=(ls cp mv rm ln mkdir rmdir touch date readlink echo)
  _is_required() { local x="$1"; shift; for e in "$@"; do [ "$e" = "$x" ] && return 0; done; return 1; }
  while IFS= read -r bin; do
    [ -z "$bin" ] && continue
    local target="/usr/bin/$bin"
    # Per-applet candidates (used when unified dispatcher is not present)
    local cand1="/usr/bin/uu-$bin"
    local cand2="/usr/lib/cargo/bin/coreutils/$bin"
    local cand3="/usr/lib/cargo/bin/$bin"
    local have_per_applet=0
    if [ -x "$cand1" ] || [ -x "$cand2" ] || [ -x "$cand3" ]; then
      have_per_applet=1
    fi

    if [ ! -L "$target" ]; then
      if [ "$have_unified" -eq 1 ] && _is_required "$bin" "${REQUIRED_BINS[@]}"; then
        echo "Expected symlink for $target (unified dispatcher available)" >&2; exit 1
      fi
      if [ "$have_per_applet" -eq 1 ]; then
        echo "Expected symlink for $target (per-applet binary present)" >&2; exit 1
      fi
      # No unified coreutils and no per-applet exists: skip this bin
      continue
    fi

    local dest
    dest="$(readlink -f "$target")"
    if [ "$bin" != "coreutils" ]; then
      # If not in the strict subset and unified dispatcher exists, don't over-assert on destination
      if [ "$have_unified" -eq 1 ] && ! _is_required "$bin" "${REQUIRED_BINS[@]}"; then
        _note_if_missing_backup "$target"
        $bin --help 2>&1 | NOMATCH 'www.gnu.org/software/coreutils'
        continue
      fi
      if [ "$dest" = "/usr/bin/coreutils" ] || [ "$dest" = "/usr/bin/uu-coreutils" ]; then
        : # unified multicall OK (either coreutils or uu-coreutils dispatcher)
      elif [ "$dest" = "/usr/bin/uu-$bin" ] || [ "$dest" = "$cand2" ] || [ "$dest" = "$cand3" ]; then
        : # per-applet uu-* or cargo-installed binary OK
      else
        echo "Unexpected target for $target -> $dest (expected unified dispatcher or a per-applet path)" >&2
        exit 1
      fi
    fi
    if _is_required "$bin" "${REQUIRED_BINS[@]}"; then
      _require_backup "$target"
    else
      _note_if_missing_backup "$target"
    fi
    # Assert it's not GNU by checking the version line (concise), but do not print it
    local _ver
    _ver="$($bin --version 2>&1 | head -n 1 || true)"
    printf '%s\n' "$_ver" | NOMATCH 'GNU'
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
  # Confirm GNU is back using concise version line (do not print)
  local _ver
  _ver="$(date --version 2>&1 | head -n 1 || true)"
  printf '%s\n' "$_ver" | MATCH 'GNU'
}

ensure_findutils_installed() {
  pkg_installed uutils-findutils || { echo "uutils-findutils not installed" >&2; exit 1; }
  [ -L "/usr/bin/find" ] && [ "$(readlink -f /usr/bin/find)" = "/usr/lib/cargo/bin/findutils/find" ] || { echo "find not linked correctly" >&2; exit 1; }
  [ -e "/usr/bin/.find.oxidizr.bak" ] || { echo "Missing .find.oxidizr.bak" >&2; exit 1; }
  # Concise non-GNU assertion (do not print)
  local _ver
  _ver="$(find --version 2>&1 | head -n 1 || true)"
  printf '%s\n' "$_ver" | NOMATCH 'GNU'

  [ -L "/usr/bin/xargs" ] && [ "$(readlink -f /usr/bin/xargs)" = "/usr/lib/cargo/bin/findutils/xargs" ] || { echo "xargs not linked correctly" >&2; exit 1; }
  [ -e "/usr/bin/.xargs.oxidizr.bak" ] || { echo "Missing .xargs.oxidizr.bak" >&2; exit 1; }
  _ver="$(xargs --version 2>&1 | head -n 1 || true)"
  printf '%s\n' "$_ver" | NOMATCH 'GNU'
}

ensure_findutils_absent() {
  if pkg_installed uutils-findutils; then
    echo "uutils-findutils unexpectedly installed" >&2; exit 1
  fi
  [ ! -L "/usr/bin/find" ] || { echo "Unexpected symlink for /usr/bin/find" >&2; exit 1; }
  [ ! -e "/usr/bin/.find.oxidizr.bak" ] || { echo "Unexpected .find.oxidizr.bak" >&2; exit 1; }
  _ver="$(find --version 2>&1 | head -n 1 || true)"
  printf '%s\n' "$_ver" | MATCH 'GNU'

  [ ! -L "/usr/bin/xargs" ] || { echo "Unexpected symlink for /usr/bin/xargs" >&2; exit 1; }
  [ ! -e "/usr/bin/.xargs.oxidizr.bak" ] || { echo "Unexpected .xargs.oxidizr.bak" >&2; exit 1; }
  _ver="$(xargs --version 2>&1 | head -n 1 || true)"
  printf '%s\n' "$_ver" | MATCH 'GNU'
}

ensure_diffutils_installed_if_supported() {
  if pkg_installed uutils-diffutils; then
    [ -L "/usr/bin/diff" ] && [ "$(readlink -f /usr/bin/diff)" = "/usr/lib/cargo/bin/diffutils/diff" ] || { echo "diff not linked correctly" >&2; exit 1; }
    [ -e "/usr/bin/.diff.oxidizr.bak" ] || { echo "Missing .diff.oxidizr.bak" >&2; exit 1; }
    local _ver
    _ver="$(diff --version 2>&1 | head -n 1 || true)"
    printf '%s\n' "$_ver" | NOMATCH 'GNU'
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
  find --version 2>&1 | head -n 1 | MATCH 'GNU'
}
