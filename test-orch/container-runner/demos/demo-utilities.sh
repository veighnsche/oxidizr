#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Demo: exercise coreutils, findutils, and sudo in container runner

Usage:
  demo-utilities.sh [--cleanup]

Notes:
 - Run this inside the container image (e.g., with host orchestrator --shell). The demo expects a
   'builder' user with NOPASSWD sudo.
 - This demo is intentionally NOT part of the YAML test suite.
 - This demo does not run 'oxidizr-arch enable/disable'; you should perform those steps manually
   if you want to compare behavior.
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

CLEANUP=${1:---no-cleanup}
RUN_ID=$(date +%s)

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "missing required tool: $1" >&2; exit 1; }
}

preflight() {
  need pacman
  need bash
  id -u builder >/dev/null 2>&1 || { echo "builder user missing; run container-runner setup first" >&2; exit 1; }
  if ! sudo -n true 2>/dev/null; then
    echo "sudo is required; ensure NOPASSWD sudo is configured (container-runner setup/users.go)" >&2
    exit 1
  fi
}

# Detect which implementation is active for core utilities and sudo
detect_coreutils_impl() {
  local line
  line=$(date --version 2>&1 | head -n1 || true)
  if echo "$line" | grep -qi "uutils"; then
    echo "uutils"
  elif echo "$line" | grep -qi "gnu coreutils"; then
    echo "gnu"
  else
    echo "unknown"
  fi
}

detect_sudo_impl() {
  local line
  line=$(sudo --version 2>&1 | head -n1 || true)
  if echo "$line" | grep -qi "sudo-rs"; then
    echo "sudo-rs"
  elif echo "$line" | grep -qi "sudo"; then
    echo "sudo"
  else
    echo "unknown"
  fi
}

run_demo() {
  local DEMO_DIR="$1"
  local SUDO_OUT_FILE="$2"
  local SUDO_MSG="${3:-hello-from-demo}-$RUN_ID"

  echo "[demo] Creating sandbox at ${DEMO_DIR}..."
  rm -rf "$DEMO_DIR"
  mkdir -p "$DEMO_DIR/sub/dir"
  printf "alpha\nBeta\nalpha\nGAmma\n" > "$DEMO_DIR/input.txt"
  printf "foo bar baz\nhello world\nfoo qux\n" > "$DEMO_DIR/words.txt"
  touch "$DEMO_DIR/sub/a" "$DEMO_DIR/sub/b" "$DEMO_DIR/sub/dir/c"
  echo "[proof][initial] tree and permissions"
  ls -l "$DEMO_DIR" || true
  ls -l "$DEMO_DIR/sub" || true
  ls -l "$DEMO_DIR/sub/dir" || true

  echo "[demo][coreutils] Basic file ops (cp/mv/ln/rm/chmod/touch/wc/head/tail/cut/tr/sort/uniq)"
  echo "+ cp '$DEMO_DIR/input.txt' '$DEMO_DIR/copy.txt'"; cp "$DEMO_DIR/input.txt" "$DEMO_DIR/copy.txt"
  echo "+ mv '$DEMO_DIR/copy.txt' '$DEMO_DIR/moved.txt'"; mv "$DEMO_DIR/copy.txt" "$DEMO_DIR/moved.txt"
  echo "+ ln -s '$DEMO_DIR/moved.txt' '$DEMO_DIR/link.txt'"; ln -s "$DEMO_DIR/moved.txt" "$DEMO_DIR/link.txt"
  echo "+ chmod 600 '$DEMO_DIR/moved.txt'"; chmod 600 "$DEMO_DIR/moved.txt"
  [[ -f "$DEMO_DIR/moved.txt" && -L "$DEMO_DIR/link.txt" ]]
  echo "[proof] ls -l for moved/link"; ls -l "$DEMO_DIR/moved.txt" "$DEMO_DIR/link.txt"
  COUNT=$(wc -l < "$DEMO_DIR/moved.txt")
  echo "[proof] wc -l moved.txt => $COUNT"
  [[ "$COUNT" -eq 4 ]]
  echo "+ head -n 2 moved.txt > head.txt"; head -n 2 "$DEMO_DIR/moved.txt" > "$DEMO_DIR/head.txt"
  echo "+ tail -n 2 moved.txt > tail.txt"; tail -n 2 "$DEMO_DIR/moved.txt" > "$DEMO_DIR/tail.txt"
  echo "+ cut/sort/uniq words.txt > first-words.counts"; cut -d ' ' -f1 "$DEMO_DIR/words.txt" | sort | uniq -c > "$DEMO_DIR/first-words.counts"
  # Expect 'foo' appears 2 times, 'hello' 1 time
  grep -Eq "^\s*2 foo$" "$DEMO_DIR/first-words.counts"
  grep -Eq "^\s*1 hello$" "$DEMO_DIR/first-words.counts"
  echo "[proof] first-words.counts"; cat "$DEMO_DIR/first-words.counts"
  echo "+ tr upper->lower moved.txt > lower.txt"; tr '[:upper:]' '[:lower:]' < "$DEMO_DIR/moved.txt" > "$DEMO_DIR/lower.txt"
  grep -q "gamma" "$DEMO_DIR/lower.txt"
  echo "[proof] grep 'gamma' lower.txt"; grep -n "gamma" "$DEMO_DIR/lower.txt"

  echo "[demo][coreutils] date and dd sanity"
  echo "+ date --version | head -1"; date --version | head -n 1 || true
  date +%Y >/dev/null 2>&1
  echo "+ printf '0123456789abcdef' | dd bs=4 count=2 of=dd.bin"; printf "0123456789abcdef" | dd bs=4 count=2 status=none of="$DEMO_DIR/dd.bin"
  [[ -s "$DEMO_DIR/dd.bin" ]]
  echo "[proof] od of dd.bin"; od -An -tx1 -N16 "$DEMO_DIR/dd.bin" | sed 's/^/  /'

  echo "[demo][findutils] find/xargs on directory tree"
  FOUND=$(find "$DEMO_DIR" -type f -name "*.txt" | wc -l)
  [[ "$FOUND" -ge 4 ]]
  # Replace spaces with underscores for files under $DEMO_DIR using find -print0 | xargs -0
  printf "%s\n" "$DEMO_DIR/space file.txt" > "$DEMO_DIR/space file.txt"
  echo "[proof] files with spaces (before)"; find "$DEMO_DIR" -type f -name "* *" -print | sed 's/^/  /'
  echo "+ find ... -print0 | xargs -0 mv name->underscored"; find "$DEMO_DIR" -type f -name "* *" -print0 | xargs -0 -I{} bash -c 'mv "$1" "${1// /_}"' _ {}
  test -f "$DEMO_DIR/space_file.txt"
  echo "[proof] files with spaces (after)"; find "$DEMO_DIR" -type f -name "* *" -print | sed 's/^/  /'


  echo "[demo][sudo] verify sudo works for builder user"
  echo "+ sudo --version | head -1"; sudo --version | head -n 1 || true
  su - builder -c 'sudo -n true'
  su - builder -c 'test "$(sudo -n id -u)" -eq 0'
  echo "[proof] before append:"; ls -l "${SUDO_OUT_FILE}" 2>/dev/null || true
  echo "+ sudo append message to ${SUDO_OUT_FILE}"; su - builder -c "sudo -n bash -c \"echo ${SUDO_MSG} >> ${SUDO_OUT_FILE}\""
  echo "[proof] tail of ${SUDO_OUT_FILE}"; tail -n 3 "${SUDO_OUT_FILE}" | sed 's/^/  /'
  grep -q "${SUDO_MSG}" "${SUDO_OUT_FILE}"
}

main() {
  preflight

  echo "[demo] Running utilities demo (coreutils/findutils/sudo) without modifying oxidizr-arch state..."
  run_demo "/tmp/oxidizr-demo" "/root/oxidizr_sudo_demo.txt" "hello-from-sudo"
  printf "[demo] Utilities demo finished. If you want to compare behavior with and without oxidizr-arch,\n      run 'oxidizr-arch enable --yes' or 'oxidizr-arch disable --yes --all' manually and re-run the demo.\n"

  # Final proof of which implementation was active during this run
  local COREUTILS_IMPL SUDO_IMPL
  COREUTILS_IMPL=$(detect_coreutils_impl)
  SUDO_IMPL=$(detect_sudo_impl)
  echo "[demo][summary] coreutils implementation detected: ${COREUTILS_IMPL} ($(date --version 2>&1 | head -n1))"
  echo "[demo][summary] sudo implementation detected: ${SUDO_IMPL} ($(sudo --version 2>&1 | head -n1))"

  if [[ "$CLEANUP" == "--cleanup" && -z "${CI:-}" ]]; then
    echo "[demo] Cleaning up..."
    echo "[proof] before cleanup: demo dir and sudo out existence"
    test -e /tmp/oxidizr-demo && echo "  exists: /tmp/oxidizr-demo" || echo "  missing: /tmp/oxidizr-demo"
    test -e /root/oxidizr_sudo_demo.txt && echo "  exists: /root/oxidizr_sudo_demo.txt" || echo "  missing: /root/oxidizr_sudo_demo.txt"
    rm -rf /tmp/oxidizr-demo || true
    rm -f /root/oxidizr_sudo_demo.txt || true
    echo "[proof] after cleanup: demo dir and sudo out existence"
    test -e /tmp/oxidizr-demo && echo "  exists: /tmp/oxidizr-demo" || echo "  removed: /tmp/oxidizr-demo"
    test -e /root/oxidizr_sudo_demo.txt && echo "  exists: /root/oxidizr_sudo_demo.txt" || echo "  removed: /root/oxidizr_sudo_demo.txt"
  fi
}

main "$@"
