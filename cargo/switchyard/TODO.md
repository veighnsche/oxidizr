# Switchyard Split & Publish Plan (TODO)

Goal: Extract `switchyard` from the monorepo into its own standalone public repository with:
- A self-contained crate ready for crates.io publish
- A hosted mdBook (GitHub Pages)
- Minimal, fast CI (lint, test, book build)
- Clean README with shields.io badges
- Optional: retain history for the `cargo/switchyard/` subtree

This document gives you:
- A decision matrix and recommended approach
- Exact commands to inventory, split, and verify
- A new-repo structure checklist
- CI and Pages setup steps
- Pre-publish checks for crates.io

---

## 0) Decision Matrix: How to Split

Pick one:

- A. History-preserving (recommended): Git filter-repo
  - Pros: clean history limited to `cargo/switchyard/`
  - Cons: requires `git-filter-repo` tool (or `git filter-branch` alternative)
- B. History-preserving (alternative): Git subtree split
  - Pros: builtin; OK for simple cases
  - Cons: less flexible than filter-repo
- C. No history: Fresh repo, copy files
  - Pros: simplest
  - Cons: no past history retained

Recommended: **A. filter-repo** if you want accurate history; otherwise **C** for speed.

---

## 1) Pre-Split Inventory (what to move, what to drop)

Run these from the monorepo root.

### 1.1 Locate all `switchyard` references outside its crate

```bash
# Find references to cargo/switchyard paths across the repo
rg -n "cargo/switchyard" -S

# Find imports or mentions of switchyard outside the crate
rg -n "\bswitchyard\b" --glob '!cargo/switchyard/**' -S
```

### 1.2 Identify switchyard-owned assets to carry over

- Keep (move into new repo):
  - `cargo/switchyard/src/`
  - `cargo/switchyard/Cargo.toml`
  - `cargo/switchyard/README.md`
  - `cargo/switchyard/book/` (mdBook sources)
  - `cargo/switchyard/SPEC/` (spec, schemas, tools used in docs/tests)
  - `cargo/switchyard/tests/` (unit/BDD as feasible without monorepo deps)
  - `cargo/switchyard/DOCS/` (if exists and is crate-specific)
  - Licensing files from monorepo root: `LICENSE`, `LICENSE-MIT`

- Consider dropping or stubbing (monorepo coupling):
  - Any tests that depend on monorepo tooling (`test-orch`, heavy Docker-based suites)
  - Monorepo-only CI jobs and guardrails unrelated to the standalone crate

### 1.3 Fix hard-coded path references before/after move

```bash
# Detect links that still reference monorepo paths
rg -n "cargo/switchyard/" cargo/switchyard/README.md cargo/switchyard/book cargo/switchyard/SPEC -S

# Detect absolute repo-root relative references in book
rg -n "\(cargo/switchyard/" cargo/switchyard/book/src -S --type md
```

Update any `cargo/switchyard/…` links to relative paths appropriate for the new repo.

---

## 2) Perform the Split

### Option A: filter-repo (history-preserving)

Prereq: `pipx install git-filter-repo` or your package manager equivalent.

```bash
# 1) Clone a fresh working copy
cd /tmp
git clone https://github.com/veighnsche/oxidizr-arch.git switchyard-split
cd switchyard-split

# 2) Filter to keep only cargo/switchyard/ (rewrite it to repo root)
# WARNING: This rewrites history in-place in your working clone.
# If using git < 2.22, install 'git-filter-repo' separately.

git filter-repo --path cargo/switchyard/ --path-rename cargo/switchyard/:

# 3) Inspect result and prune unwanted files
ls -la

# 4) Copy top-level licenses from the original repo (if not present)
# Alternative: stage beforehand in a throwaway branch and then filter.

# 5) Create a new GitHub repo (empty) and push
NEW_REPO_SSH=git@github.com:veighnsche/switchyard.git

git remote remove origin || true
git remote add origin "$NEW_REPO_SSH"

git push -u origin HEAD:main
```

### Option B: subtree split (history-preserving)

```bash
# From your monorepo checkout (not a bare repo):
cd /home/vince/Projects/oxidizr-arch

git subtree split --prefix=cargo/switchyard -b switchyard-split

# Create a fresh repo directory and push the split branch into it
mkdir -p /tmp/switchyard-new && cd /tmp/switchyard-new
git init -b main

git remote add source /home/vince/Projects/oxidizr-arch
git pull source switchyard-split

NEW_REPO_SSH=git@github.com:veighnsche/switchyard.git
git remote add origin "$NEW_REPO_SSH"
git push -u origin main
```

### Option C: Fresh repo (no history)

```bash
mkdir -p /tmp/switchyard && cd /tmp/switchyard
git init -b main

# Copy only the needed tree
rsync -a --exclude target \
  /home/vince/Projects/oxidizr-arch/cargo/switchyard/ ./

# Add licenses from monorepo root
cp /home/vince/Projects/oxidizr-arch/LICENSE ./LICENSE
cp /home/vince/Projects/oxidizr-arch/LICENSE-MIT ./LICENSE-MIT

# Create repo on GitHub and push
NEW_REPO_SSH=git@github.com:veighnsche/switchyard.git
git remote add origin "$NEW_REPO_SSH"
git add .
git commit -m "Initial import: switchyard crate, book, SPEC"
git push -u origin main
```

### Optional: keep monorepo tracking via submodule

From monorepo root:

```bash
cd /home/vince/Projects/oxidizr-arch

git rm -r cargo/switchyard

git submodule add -b main git@github.com:veighnsche/switchyard.git cargo/switchyard
git commit -m "Switch cargo/switchyard to submodule"
```

---

## 3) New Repo Structure Checklist

In the new repository (root):

```
LICENSE
LICENSE-MIT
README.md
Cargo.toml
src/
book/
  book.toml
  src/
SPEC/
.github/
  workflows/
    ci.yml
    book.yml
```

- `Cargo.toml` metadata (already aligned in monorepo crate):
  - `name = "switchyard"`
  - `homepage = "https://<org>.github.io/<repo>/"` (GitHub Pages URL)
  - `documentation = "https://docs.rs/switchyard"`
  - `readme = "README.md"`
  - `keywords`, `categories`, `rust-version`, `exclude`
  - `[badges]` and `[package.metadata.docs.rs]`
- `README.md`:
  - Top links to mdBook and docs.rs
  - Shields badges (see §5)
- `book/` contains mdBook sources; `book/book` is the built site by CI only

---

## 4) CI & Pages (in the new repo)

### 4.1 CI: Minimal fast checks (`.github/workflows/ci.yml`)

```yaml
name: CI
on:
  push: { branches: [ main ] }
  pull_request: { branches: [ main ] }
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy
      - run: cargo fmt -- --check
      - run: cargo clippy --all-targets -- -D warnings
  test:
    runs-on: ubuntu-latest
    needs: lint
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - run: cargo test --all-features
```

### 4.2 mdBook build & deploy (`.github/workflows/book.yml`)

(Uses `peaceiris/actions-mdbook` and `peaceiris/actions-gh-pages`.)

```yaml
name: mdBook
on:
  push: { branches: [ main ] }
  pull_request: { branches: [ main ] }
jobs:
  book-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: peaceiris/actions-mdbook@v2
        with: { mdbook-version: latest }
      - run: mdbook build book
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: mdbook-site
          path: book/book
  book-deploy:
    runs-on: ubuntu-latest
    needs: book-build
    if: github.ref == 'refs/heads/main'
    permissions: { contents: write }
    steps:
      - uses: actions/checkout@v4
      - uses: peaceiris/actions-mdbook@v2
        with: { mdbook-version: latest }
      - run: mdbook build book
      - uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: book/book
          force_orphan: true
```

### 4.3 Enable GitHub Pages

- Settings → Pages → Branch: `gh-pages` (created by deploy action) → `/ (root)`
- The `homepage` in `Cargo.toml` should match `https://<org>.github.io/<repo>/`

---

## 5) README badges (shields.io)

Add these near the top of `README.md` in the new repo:

```md
[![Crates.io](https://img.shields.io/crates/v/switchyard.svg)](https://crates.io/crates/switchyard)
[![docs.rs](https://img.shields.io/docsrs/switchyard)](https://docs.rs/switchyard)
[![CI](https://github.com/<org>/<repo>/actions/workflows/ci.yml/badge.svg)](https://github.com/<org>/<repo>/actions/workflows/ci.yml)
[![mdBook](https://img.shields.io/badge/book-mdBook-blue)](https://<org>.github.io/<repo>/)
[![License: Apache-2.0/MIT](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)](./LICENSE)
[![MSRV 1.75+](https://img.shields.io/badge/MSRV-1.75%2B-informational)](./Cargo.toml)
```

Replace `<org>/<repo>` with your GitHub org/repo (e.g., `veighnsche/switchyard`).

---

## 6) Pre-publish Checks (new repo)

```bash
# 1) Sanity: build & test
cargo check
cargo test --all-features

# 2) Docs: rustdoc and docs.rs compatibility
RUSTDOCFLAGS="--cfg docsrs" cargo doc --no-deps --all-features

# 3) Book: build locally
mdbook build book
# Optional: linkcheck (if you add mdbook-linkcheck)
# cargo install mdbook-linkcheck
# mdbook build -d book/book --dest-dir book && mdbook-linkcheck -d book/book

# 4) Package inspection: what crates.io will see
cargo package --list
cargo publish --dry-run
```

Ensure:
- `Cargo.toml` has proper `homepage`, `documentation`, `readme`, `categories`, `keywords`, `rust-version`, `exclude`
- `LICENSE` and `LICENSE-MIT` exist at repo root
- README links resolve (mdBook and docs.rs)

Optional extra gates:
```bash
# cargo-deny for licenses/vulns (optional)
cargo install cargo-deny
cargo deny check
```

---

## 7) Link & Path Hygiene

Scan and fix references that still point back to the monorepo:

```bash
# In new repo root
rg -n "cargo/switchyard/" -S
rg -n "oxidizr-arch" -S
rg -n "../.." book/src -S --type md
```

Replace with relative links or new repo URLs.

---

## 8) Publishing

```bash
# Final packaging check
cargo publish --dry-run

# Publish
cargo publish
```

After publishing:
- Verify the crate on crates.io shows README and links.
- Confirm `homepage` opens the mdBook (GitHub Pages) site.
- Confirm docs.rs generated docs (badge green).

---

## 9) Post-Split: Keep Monorepo in Sync (if needed)

If you chose to submodule the new repo back into the monorepo:
- Update monorepo CI to skip submodule contents (if necessary) or keep minimal checks.
- Document contribution flow: PRs against the new repo for library changes.

---

## 10) Quick Commands Recap

- Inventory references: `rg -n "cargo/switchyard" -S`
- Extract history (filter-repo): `git filter-repo --path cargo/switchyard/ --path-rename cargo/switchyard/:`
- Push new repo: `git push -u origin main`
- Add submodule back: `git submodule add -b main <git url> cargo/switchyard`
- Build/test: `cargo check && cargo test`
- Book: `mdbook build book`
- Package/publish: `cargo package --list && cargo publish --dry-run && cargo publish`

---

## 11) Known Monorepo Couplings to Revisit

- Some tests and docs reference monorepo paths: update to new repo
- Any BDD/E2E suites using orchestrators or Docker matrices should be moved to a separate repo or deleted for the standalone library
- The mdBook cites SPEC and INVENTORY; these live under `SPEC/` and must move with the crate

Done. Follow sections 2–6 sequentially to stand up the new `switchyard` repository with its own CI and mdBook publishing, then publish to crates.io.
