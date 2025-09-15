# Plan: Combinatorial Test Matrix for oxidizr-deb

## 1) Goal

Ensure broad coverage across flags and environment conditions without exploding test runtime.

## 2) Parameters

- Package: {coreutils, findutils, sudo}
- Mode: {dry_run, commit}
- Root: {fakeroot, system-root (skip in CI)}
- Fetch channel: {stable, latest}
- Offline/local artifact: {none, provided}
- Flags: {assume_yes ∈ {0,1}, use_alternatives ∈ {0,1}, use_divert ∈ {0,1}}
- FS condition: {same_fs, exdev}
- PM locks: {present, absent}
- Rescue available: {true, false}
- Sudo replacement perms: {ok(4755 root:root), bad} (only when package=sudo)

## 3) Constraints (prune invalid combos)

- `use_alternatives` only meaningful for coreutils and findutils; for sudo, mark N/A.
- `use_divert` not valid in pure dry-run (or becomes a no-op mock); mark as xfail for dry-run path.
- `sudo perms` only relevant when package=sudo.
- `exdev` only meaningful on rustify operations (not restore-only scenarios).
- `offline/local` requires a provided artifact; otherwise invalid.

## 4) Strategy

- Pairwise (AllPairs) baseline; 3-wise for riskier intersections: {package, fs_condition, mode}, {package, pm_locks, mode}, {package, fetch_channel, offline/local}.
- Generate matrix via small Rust or Python utility:
  - Input: parameter domains + constraints.
  - Output: JSON/CSV consumed by the BDD runner to parametrize scenarios (tags → args/env setup).
- Tag mapping: scenario outlines with examples; each row becomes an example.

## 5) Tooling

- Rust bin `scripts/gen_matrix.rs` (or Python `scripts/gen_matrix.py`) using a pairwise library (or custom greedy algorithm).
- Emit `target/test-matrix.json` for CI consumption.

## 6) Execution in CI

- Load `test-matrix.json` and drive `cucumber`/`cargo test` via env vars per example row.
- Parallelize by chunking examples across CI shards.

## 7) Acceptance

- Matrix covers every parameter at least pairwise; critical triples covered.
- Constraints enforced; no invalid combos executed.
- CI job times kept reasonable (<10 min for oxidizr-deb suite on typical runners).
