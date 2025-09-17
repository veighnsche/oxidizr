#!/usr/bin/env bash
set -euo pipefail

# 1_split_dry_run.sh
# Analyze and dry-run the split of a subdirectory using git-filter-repo
# - Creates a fresh disposable clone (to satisfy git-filter-repo safety checks)
# - Runs --analyze and a --dry-run filter
# - Does not push or modify the original repository
#
# Environment variables (can also be set via flags):
#   MONOREPO       Path to the original monorepo (default: /home/vince/Projects/oxidizr)
#   WORKDIR        Working directory to use (default: mktemp -d)
#   SUBDIR         Subdirectory to extract (default: cargo/oxidizr-arch)
#   TARGET_BRANCH  Target branch name for the new repo (default: main)
#   PATHS_FILE     Optional file for --paths-from-file (to handle historical renames)
#
# Flags:
#   -m <path>   Set MONOREPO
#   -w <path>   Set WORKDIR (will be created if not exists)
#   -s <path>   Set SUBDIR
#   -b <name>   Set TARGET_BRANCH
#   -p <file>   Set PATHS_FILE (use paths-from-file instead of --subdirectory-filter)
#   -h          Show help

usage() {
  cat <<EOF
Usage: $0 [-m MONOREPO] [-w WORKDIR] [-s SUBDIR] [-b TARGET_BRANCH] [-p PATHS_FILE]

Analyze and perform a dry run of splitting a subdirectory with git-filter-repo.

ENV defaults:
  MONOREPO      := "+${MONOREPO:-/home/vince/Projects/oxidizr}+"
  WORKDIR       := new temp dir (mktemp)
  SUBDIR        := "+${SUBDIR:-cargo/oxidizr-arch}+"
  TARGET_BRANCH := "+${TARGET_BRANCH:-main}+"
  PATHS_FILE    := unset
EOF
}

# Defaults
MONOREPO="${MONOREPO:-/home/vince/Projects/oxidizr}"
WORKDIR="${WORKDIR:-}"
SUBDIR="${SUBDIR:-cargo/oxidizr-arch}"
TARGET_BRANCH="${TARGET_BRANCH:-main}"
PATHS_FILE="${PATHS_FILE:-}"

while getopts ":m:w:s:b:p:h" opt; do
  case "$opt" in
    m) MONOREPO="$OPTARG" ;;
    w) WORKDIR="$OPTARG" ;;
    s) SUBDIR="$OPTARG" ;;
    b) TARGET_BRANCH="$OPTARG" ;;
    p) PATHS_FILE="$OPTARG" ;;
    h) usage; exit 0 ;;
    *) usage; exit 2 ;;
  esac
done

err() { echo "[ERROR] $*" >&2; }
log() { echo "[INFO] $*"; }

need_cmd() { command -v "$1" >/dev/null 2>&1 || { err "Missing required command: $1"; exit 1; }; }

need_cmd git
need_cmd git-filter-repo || true # Arch installs git-filter-repo; we will invoke via 'git filter-repo'

if [[ ! -d "$MONOREPO/.git" ]]; then
  err "MONOREPO does not look like a git repo: $MONOREPO"
  exit 1
fi

if [[ -z "$WORKDIR" ]]; then
  WORKDIR="$(mktemp -d -t oxidizr-arch-split-XXXXXX)"
else
  mkdir -p "$WORKDIR"
fi

log "Using WORKDIR: $WORKDIR"

# Fresh clone (avoid local hardlinks/shared objects)
log "Cloning fresh copy (git clone --no-local) ..."
 git clone --no-local "$MONOREPO" "$WORKDIR/mono" >/dev/null
 cd "$WORKDIR/mono"

# Ensure not shallow/partial
if [[ -f .git/shallow ]]; then
  log "Repository is shallow; unshallowing ..."
  git fetch --unshallow --tags >/dev/null
fi

# Analyze history to aid path decisions
log "Running: git filter-repo --analyze"
 git filter-repo --analyze
log "Analysis written under .git/filter_repo/analysis"

# Dry-run filter: prefer paths file if provided
if [[ -n "$PATHS_FILE" ]]; then
  if [[ ! -f "$PATHS_FILE" ]]; then
    err "PATHS_FILE not found: $PATHS_FILE"
    exit 1
  fi
  log "Dry-run with --paths-from-file: $PATHS_FILE"
  git filter-repo --dry-run --paths-from-file "$PATHS_FILE"
else
  log "Dry-run with --subdirectory-filter: $SUBDIR"
  git filter-repo --dry-run --subdirectory-filter "$SUBDIR"
fi

log "Dry-run complete. No changes were made to the original repo."
log "Next: review .git/filter_repo/analysis, and then run scripts/2_spli_commit.sh to perform the actual rewrite and push."
