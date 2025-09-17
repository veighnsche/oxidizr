#!/usr/bin/env bash
set -euo pipefail

# 2_spli_commit.sh
# Perform the actual history rewrite for a subdirectory using git-filter-repo,
# push to a new empty repository, and (optionally) convert the monorepo folder
# into a Git submodule pointing at that repository.
#
# This script operates in a fresh disposable clone to satisfy git-filter-repo
# safety checks. The original monorepo is not modified until you optionally
# perform the submodule step.
#
# Environment variables (can also be set via flags):
#   MONOREPO       Path to the original monorepo (default: /home/vince/Projects/oxidizr)
#   WORKDIR        Working directory to use (default: mktemp -d)
#   SUBDIR         Subdirectory to extract (default: cargo/oxidizr-arch)
#   NEW_REPO_SSH   SSH URL of the new empty repo (default: git@github.com:veighnsche/oxidizr-arch.git)
#   TARGET_BRANCH  Target branch name for the new repo (default: main)
#   PATHS_FILE     Optional file for --paths-from-file (to handle historical renames)
#   DO_SUBMODULE   If set to 1, will offer to convert the monorepo folder to a submodule (default: 0)
#   TRACK_BRANCH   If set to 1, submodule will track the target branch (uses -b) (default: 0)
#   YES            If set to 1, skip interactive confirmations (default: 0)
#
# Flags:
#   -m <path>   Set MONOREPO
#   -w <path>   Set WORKDIR (will be created if not exists)
#   -s <path>   Set SUBDIR
#   -r <url>    Set NEW_REPO_SSH
#   -b <name>   Set TARGET_BRANCH
#   -p <file>   Set PATHS_FILE (use paths-from-file instead of --subdirectory-filter)
#   -S          Set DO_SUBMODULE=1 (attempt submodule conversion in monorepo)
#   -t          Set TRACK_BRANCH=1 (submodule add -b TARGET_BRANCH)
#   -y          Set YES=1 (non-interactive)
#   -h          Show help

usage() {
  cat <<EOF
Usage: $0 [-m MONOREPO] [-w WORKDIR] [-s SUBDIR] [-r NEW_REPO_SSH] [-b TARGET_BRANCH] \
          [-p PATHS_FILE] [-S] [-t] [-y]

Rewrite subdir history with git-filter-repo in a fresh clone, push to new repo,
optionally replace the directory with a submodule in the monorepo.

ENV defaults:
  MONOREPO      := "+${MONOREPO:-/home/vince/Projects/oxidizr}+"
  WORKDIR       := new temp dir (mktemp)
  SUBDIR        := "+${SUBDIR:-cargo/oxidizr-arch}+"
  NEW_REPO_SSH  := "+${NEW_REPO_SSH:-git@github.com:veighnsche/oxidizr-arch.git}+"
  TARGET_BRANCH := "+${TARGET_BRANCH:-main}+"
  PATHS_FILE    := unset
  DO_SUBMODULE  := "+${DO_SUBMODULE:-0}+" (0/1)
  TRACK_BRANCH  := "+${TRACK_BRANCH:-0}+" (0/1)
  YES           := "+${YES:-0}+" (0/1)
EOF
}

# Defaults
MONOREPO="${MONOREPO:-/home/vince/Projects/oxidizr}"
WORKDIR="${WORKDIR:-}"
SUBDIR="${SUBDIR:-cargo/oxidizr-arch}"
NEW_REPO_SSH="${NEW_REPO_SSH:-git@github.com:veighnsche/oxidizr-arch.git}"
TARGET_BRANCH="${TARGET_BRANCH:-main}"
PATHS_FILE="${PATHS_FILE:-}"
DO_SUBMODULE="${DO_SUBMODULE:-0}"
TRACK_BRANCH="${TRACK_BRANCH:-0}"
YES="${YES:-0}"

while getopts ":m:w:s:r:b:p:Styh" opt; do
  case "$opt" in
    m) MONOREPO="$OPTARG" ;;
    w) WORKDIR="$OPTARG" ;;
    s) SUBDIR="$OPTARG" ;;
    r) NEW_REPO_SSH="$OPTARG" ;;
    b) TARGET_BRANCH="$OPTARG" ;;
    p) PATHS_FILE="$OPTARG" ;;
    S) DO_SUBMODULE="1" ;;
    t) TRACK_BRANCH="1" ;;
    y) YES="1" ;;
    h) usage; exit 0 ;;
    *) usage; exit 2 ;;
  esac
done

err() { echo "[ERROR] $*" >&2; }
log() { echo "[INFO] $*"; }
confirm() {
  if [[ "$YES" == "1" ]]; then return 0; fi
  read -r -p "$1 [y/N] " ans
  [[ "$ans" == "y" || "$ans" == "Y" ]]
}

need_cmd() { command -v "$1" >/dev/null 2>&1 || { err "Missing required command: $1"; exit 1; }; }

need_cmd git
need_cmd tar
need_cmd sed || true
if ! git filter-repo -h >/dev/null 2>&1; then
  err "'git filter-repo' is not available. Install git-filter-repo (e.g., 'sudo pacman -S git-filter-repo')."
  exit 1
fi

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

log "This will rewrite history in a disposable clone and push to: $NEW_REPO_SSH (branch: $TARGET_BRANCH)."
confirm "Proceed with rewrite and push?" || { log "Aborted."; exit 1; }

# Fresh clone (avoid local hardlinks/shared objects)
log "Cloning fresh copy (git clone --no-local) ..."
 git clone --no-local "$MONOREPO" "$WORKDIR/mono" >/dev/null
 cd "$WORKDIR/mono"

# Ensure not shallow/partial
if [[ -f .git/shallow ]]; then
  log "Repository is shallow; unshallowing ..."
  git fetch --unshallow --tags >/dev/null
fi

# Perform the rewrite
if [[ -n "$PATHS_FILE" ]]; then
  if [[ ! -f "$PATHS_FILE" ]]; then
    err "PATHS_FILE not found: $PATHS_FILE"
    exit 1
  fi
  log "Rewriting with --paths-from-file: $PATHS_FILE"
  git filter-repo --paths-from-file "$PATHS_FILE"
else
  log "Rewriting with --subdirectory-filter: $SUBDIR"
  git filter-repo --subdirectory-filter "$SUBDIR"
fi

# Remote handling
if git remote | grep -q '^origin$'; then
  log "Renaming existing 'origin' to 'source' to avoid accidental push back ..."
  git remote rename origin source
fi
log "Adding new origin: $NEW_REPO_SSH"
 git remote add origin "$NEW_REPO_SSH"
 git remote -v

# Ensure branch name, then push
CURRENT_BRANCH=$(git symbolic-ref --short HEAD || echo "")
if [[ "$CURRENT_BRANCH" != "$TARGET_BRANCH" ]]; then
  log "Renaming branch '$CURRENT_BRANCH' -> '$TARGET_BRANCH'"
  git branch -M "$TARGET_BRANCH"
fi

log "Pushing to new repo ..."
 git push -u origin "$TARGET_BRANCH"
log "Push complete. Verify on GitHub: $NEW_REPO_SSH"

# Optional: convert monorepo folder to submodule
if [[ "$DO_SUBMODULE" == "1" ]]; then
  log "Preparing to convert '$SUBDIR' to a submodule in $MONOREPO"
  if [[ ! -d "$MONOREPO/$SUBDIR" ]]; then
    err "Path does not exist in monorepo: $MONOREPO/$SUBDIR"
    exit 1
  fi
  confirm "Proceed with submodule conversion in monorepo?" || { log "Skipped submodule step."; exit 0; }

  BACKUP_DIR=/tmp/oxidizr-backups
  mkdir -p "$BACKUP_DIR"
  TS=$(date +%Y%m%d-%H%M%S)
  BACKUP_TGZ="$BACKUP_DIR/oxidizr-arch-pre-submodule-$TS.tgz"

  log "Creating backup tarball: $BACKUP_TGZ"
   tar -C "$MONOREPO" -czf "$BACKUP_TGZ" "$SUBDIR"

  log "Removing tracked directory and committing ..."
   cd "$MONOREPO"
   git rm -r "$SUBDIR"
   git commit -m "chore: remove embedded $(basename "$SUBDIR") in preparation for submodule"

  log "Adding submodule ..."
   if [[ "$TRACK_BRANCH" == "1" ]]; then
     git submodule add -b "$TARGET_BRANCH" "$NEW_REPO_SSH" "$SUBDIR"
   else
     git submodule add "$NEW_REPO_SSH" "$SUBDIR"
   fi
   git commit -m "feat: add $(basename "$SUBDIR") as a git submodule"

  log "Submodule added. Remember to push monorepo changes (ideally via a PR):"
  echo "  cd '$MONOREPO' && git push"
fi

log "All done."
