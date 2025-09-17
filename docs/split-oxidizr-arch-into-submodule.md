# Split `cargo/oxidizr-arch/` into its own repository and add back as a submodule (verified, safe, step-by-step)

This guide safely extracts the `cargo/oxidizr-arch/` subdirectory (with full history) into a new repository at `git@github.com:veighnsche/oxidizr-arch.git`, then replaces the directory in this monorepo with a Git submodule pointing to that new repo.

All steps are designed to be repeatable, auditable, and minimally risky.

- Target subdirectory: `cargo/oxidizr-arch/`
- New repo (empty): `git@github.com:veighnsche/oxidizr-arch.git`
- Monorepo path: `/home/vince/Projects/oxidizr`
- Assumed default branch name: `main` (adjust if your default is different)

Helper scripts included in this repository (see `scripts/`):

- `scripts/1_split_dry_run.sh` — performs a safe, disposable-clone analysis and dry run
- `scripts/2_spli_commit.sh` — performs the actual rewrite and push to the new repo

Sources (authoritative):

- GitHub Docs — Splitting a subfolder out into a new repository: <https://docs.github.com/en/get-started/using-git/splitting-a-subfolder-out-into-a-new-repository>
- git-filter-repo (official) — How to use, prerequisites, manual: <https://github.com/newren/git-filter-repo>
  - User manual: <https://htmlpreview.github.io/?https://github.com/newren/git-filter-repo/blob/docs/html/git-filter-repo.html>
- Pro Git (Submodules): <https://git-scm.com/book/en/v2/Git-Tools-Submodules>
- GitHub Blog (Submodules overview): <https://github.blog/open-source/git/working-with-submodules/>

## 0) Prerequisites and safety

- Confirm clean working tree in monorepo:

  ```bash
  git -C /home/vince/Projects/oxidizr status
  ```

- Versions (per git-filter-repo prerequisites):

  ```bash
  git --version        # >= 2.36.0 (required by git-filter-repo README)
  python3 --version    # >= 3.6
  ```

- Avoid shallow or partial clones for filtering. Ensure full history is present:

  ```bash
  # If your repo is shallow, unshallow it first
  git fetch --unshallow --tags
  # Avoid partial clone options like --filter=blob:none for the filtering step
  ```

- Install git-filter-repo (pick one):
  - Via pacman (Arch Linux):

    ```bash
    sudo pacman -S git-filter-repo
    # Verify installation & version on Arch
    pacman -Qi git-filter-repo | sed -n '1,20p'
    command -v git-filter-repo
    git filter-repo -h | head -n 5
    ```

    Arch package page: <https://archlinux.org/packages/extra/any/git-filter-repo/>

    Note (Arch): The script installs to `/usr/bin/git-filter-repo`. You should invoke it via Git’s subcommand dispatcher as `git filter-repo` (as used below), which ensures the binary on your PATH (from pacman) is used.

  - Via pipx (recommended):

    ```bash
    pipx install git-filter-repo
    ```

  - Via pip (user):

    ```bash
    pip install --user git-filter-repo
    ```

  - Or place the script in PATH (see README/INSTALL): <https://github.com/newren/git-filter-repo>

Tip: We will operate on a fresh temporary clone to avoid using `--force` and to keep the original repo untouched, as recommended by the docs.

## 1) Prepare a fresh temporary clone for filtering

```bash
# Variables
export MONOREPO=/home/vince/Projects/oxidizr
export WORKDIR=/tmp/oxidizr-arch-split
export SUBDIR=cargo/oxidizr-arch
export NEW_REPO_SSH=git@github.com:veighnsche/oxidizr-arch.git
export TARGET_BRANCH=main   # change if your target default branch is different

# Create temp work area
mkdir -p "$WORKDIR"

# Clone the monorepo freshly (no shared objects to keep things isolated)
# Important: use --no-local to avoid shared-object optimization in local clones
git clone --no-local "$MONOREPO" "$WORKDIR/mono"
cd "$WORKDIR/mono"

# Verify branch
git symbolic-ref --short HEAD || true
```

Note (fish shell): replace the `export` lines with `set -x` syntax, e.g.

```fish
set -x MONOREPO /home/vince/Projects/oxidizr
set -x WORKDIR /tmp/oxidizr-arch-split
set -x SUBDIR cargo/oxidizr-arch
set -x NEW_REPO_SSH git@github.com:veighnsche/oxidizr-arch.git
set -x TARGET_BRANCH main
```

Optional: ensure the temporary clone has no untracked refs or stashes.

## 2) Filter the history to only `cargo/oxidizr-arch/`

Following GitHub Docs and git-filter-repo manual, use `--subdirectory-filter` to make the subfolder the new repo root and preserve history.
Note: `--subdirectory-filter DIRECTORY` is equivalent to `--path DIRECTORY/ --path-rename DIRECTORY/:` (per `git filter-repo --help`).

Recommended preliminary steps:

```bash
# 2.1) Analyze history (reports useful for path/rename detection)
git filter-repo --analyze
# Report will be written to: .git/filter_repo/analysis (refuses if already exists)

# 2.2) Optional dry run (no changes are made to the repo)
git filter-repo --dry-run --subdirectory-filter "$SUBDIR"
```

Optional: If the subdirectory was renamed historically, prefer a `--paths-from-file` approach to include all historical paths and re-root them to the project root. Create a file (e.g. `$WORKDIR/paths.txt`) like:

```
# Each line selects and optionally renames a path. '==>' denotes rename target.
# Use 'literal:' (default) for exact path matching.
literal:cargo/oxidizr-arch/ ==> 
# If there were historical names, add them too, all renamed to the root:
# literal:old-name/ ==> 
# literal:older-name/ ==> 
```

Then run (dry-run first):

```bash
git filter-repo --dry-run --paths-from-file "$WORKDIR/paths.txt"
```

When satisfied, proceed to actually rewrite:

```bash
git filter-repo --subdirectory-filter "$SUBDIR"
# or, if using a paths file to capture historical renames
# git filter-repo --paths-from-file "$WORKDIR/paths.txt"
```

Notes:

- If git-filter-repo complains about safety checks, ensure you are in a fresh clone (as above). Only if absolutely needed, you may add `--force` after verifying you are in a disposable clone.
- If the folder was renamed in history, you can include multiple paths with `--path` flags instead, or use `--path-rename`; see the user manual for details.

Sanity checks after filtering:

```bash
# Should list only files that were formerly under the subdir
git ls-tree -r --name-only HEAD | sed -n '1,50p'

# Inspect history scope
git log --oneline --graph --decorate --stat | sed -n '1,100p'
```

## 3) Point the filtered repo at the new empty GitHub repository

We avoid accidental pushes back to the original remote:

```bash
# If a remote named origin exists from the source clone, rename or remove it
if git remote | grep -q '^origin$'; then
  git remote rename origin source
fi

# Add the new remote as origin
git remote add origin "$NEW_REPO_SSH"

git remote -v
```

Create the branch if needed and push. For a brand-new empty repo, pushing `HEAD` to `main` is typical:

```bash
# Ensure the branch name is what you want
CURRENT_BRANCH=$(git symbolic-ref --short HEAD)
if [ "$CURRENT_BRANCH" != "$TARGET_BRANCH" ]; then
  git branch -M "$TARGET_BRANCH"
fi

# First push to the new repository
git push -u origin "$TARGET_BRANCH"

# Optional: Only push tags you want to keep (generally avoid mass-pushing old tags)
# git tag
# git push origin <tagname>
```

Verification on the new remote:

- Browse `https://github.com/veighnsche/oxidizr-arch` and confirm files look correct.
- Verify commit history only includes changes related to `cargo/oxidizr-arch/`.

## 4) Replace the embedded folder in the monorepo with a submodule

We’ll back up the existing directory, remove it from the monorepo, and add the submodule pointing to the new repo.

```bash
# Backup the current embedded directory (outside the repo)
mkdir -p /tmp/oxidizr-backups
 tar -C "$MONOREPO" -czf /tmp/oxidizr-backups/oxidizr-arch-pre-submodule.tgz "$SUBDIR"

# In the monorepo, remove the tracked directory (without deleting the backup)
cd "$MONOREPO"
 git rm -r "$SUBDIR"
 git commit -m "chore: remove embedded oxidizr-arch in preparation for submodule"

# Add the submodule at the same path
# Option A (pin to a specific commit; reproducible):
 git submodule add "$NEW_REPO_SSH" "$SUBDIR"
# Option B (track a branch; less reproducible, but auto-updatable):
# git submodule add -b "$TARGET_BRANCH" "$NEW_REPO_SSH" "$SUBDIR"

# Commit submodule metadata (.gitmodules) and the gitlink
 git status
 git commit -m "feat: add oxidizr-arch as a git submodule"

# Push monorepo changes (on a feature branch + PR recommended)
 git push
```

Post-steps for developers/CI:

```bash
git submodule update --init --recursive
```

If you had uncommitted local changes in `cargo/oxidizr-arch/`, re-apply them inside the submodule repository and commit/push within the submodule, then update the monorepo to point to that new submodule commit.

## 5) Validation checklist

- New repo contains only `oxidizr-arch` files and complete related history.
- Monorepo now contains `.gitmodules` and shows `cargo/oxidizr-arch` as a gitlink (not regular files):

  ```bash
  git ls-tree HEAD cargo/oxidizr-arch
  ```

- Fresh clone works correctly:

  ```bash
  git clone git@github.com:veighnsche/oxidizr.git
  cd oxidizr
  git submodule update --init --recursive
  ```

## 6) Rollback plan

- If anything looks wrong in the filtered repo, delete the temporary work dir and repeat filtering. If you already pushed to the new GitHub repo, you can delete that repo (if allowed) and recreate it, or force-push a corrected filtered history (since it’s a new, owned repo).
- In the monorepo, the submodule switch is confined to one or two commits. To rollback, revert those commits or reset the branch to the prior commit. You also have a tarball backup at `/tmp/oxidizr-backups/oxidizr-arch-pre-submodule.tgz`.

## 7) Notes and tips

- If the subdirectory was renamed in history, read the git-filter-repo manual and consider `--path` with multiple instances (one per historical path) or use `--path-rename` rules to preserve earlier history across renames.
- Prefer doing the submodule replacement on a feature branch and landing via PR for review.
- Consider whether you want submodules pinned (deterministic builds) or branch-tracking (more convenience). Pro Git discusses tradeoffs.

## 8) Troubleshooting: "expected freshly packed repo"

If `git filter-repo` aborts with a message like:

> Aborting: Refusing to destructively overwrite repo history since this does not look like a fresh clone. (expected freshly packed repo)

Try the following, in the disposable temporary clone:

```bash
# Ensure the clone avoided local hardlinks/shared objects
# (This guide uses: git clone --no-local)

# If necessary, repack & prune to make the repo look freshly packed
git reflog expire --expire=now --all
git gc --prune=now --aggressive
# or
git repack -Ad

# Retry the filter
git filter-repo --subdirectory-filter "$SUBDIR"

# As a last resort (only in a disposable clone), use --force
# after double-checking you are not on a shared or important clone
# git filter-repo --subdirectory-filter "$SUBDIR" --force
```

Background:

- git-filter-repo strongly prefers operating on fresh clones to minimize risk. See the README and related discussions in the project’s issue tracker.
- Using `git clone --no-local` is often sufficient to satisfy safety checks.

## References

- GitHub Docs — Splitting a subfolder: <https://docs.github.com/en/get-started/using-git/splitting-a-subfolder-out-into-a-new-repository>
- newren/git-filter-repo (README & manual): <https://github.com/newren/git-filter-repo>
  - Alternate manual (mankier): <https://www.mankier.com/1/git-filter-repo>
- Pro Git — Submodules: <https://git-scm.com/book/en/v2/Git-Tools-Submodules>
- GitHub Blog — Working with submodules: <https://github.blog/open-source/git/working-with-submodules/>
