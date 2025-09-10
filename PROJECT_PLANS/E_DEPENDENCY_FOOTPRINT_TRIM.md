# Stream E — Dependency Footprint Trim

## 1) Scope

- Replace `which` crate usage with a small internal PATH search.
- Optionally feature-gate heavier deps.

Touched modules (validated):

- `src/system/worker/packages.rs` (uses `which::which` today)
- Callers via `Worker.which(...)` in experiments will continue to function
- `Cargo.toml` (feature flags)

## Reuse existing infrastructure

- Centralize all PATH lookups via `Worker.which()` (in `src/system/worker/fs_ops.rs`). Do not call `which::which` directly from other modules.
- Implement the internal `path_search` utility behind `Worker.which()` so callers in `experiments/*` and `system/worker/*` remain unchanged.
- If feature-gating, place flags in `Cargo.toml` and guard only the implementation behind `Worker.which()`; do not fork call sites.

## Quality Requirements (Lean & Safety)

- Lean
  - Single PATH search implementation lives behind `Worker.which()`; all call sites defer to it.
  - Feature-gate the external `which` crate and swap to an internal `path_search` without changing callers.
  - No duplicated helpers in other modules; avoid shelling out for `which`.
- Safety
  - Deterministic PATH resolution that mirrors standard semantics (respects `PATH`, executable bit, symlink handling).
  - Robustness against edge cases (relative paths, `.` and `..`, empty path entries, permission errors) with clear errors.
  - Comprehensive tracing/audit around important resolution steps when helpful for debugging (debug-level only).

## Module File Structure Blueprint

- Extend existing modules
  - `src/system/worker/fs_ops.rs`
    - `fn path_search(name: &str) -> Result<Option<PathBuf>>` — internal utility implementing PATH scanning
    - `pub fn which(&self, name: &str) -> Result<Option<PathBuf>>` — calls `path_search` (feature-gated fallback to external crate when enabled)
  - `Cargo.toml`
    - `[features]` section e.g. `which-crate = []` (default on), `internal-which = []` (alternate); or invert default as desired
    - conditional deps for `which` crate
- Tests
  - Unit corpus comparing `path_search` vs `which` results across typical cases
  - Regression tests for edge cases: missing exec bit, empty `PATH` segments, relative vs absolute, symlinks

## 2) Rationale & Safety Objectives

- Reduce trusted code base size while preserving behavior.

## 3) Architecture & Design

- Introduce `path_search` utility to scan PATH for executables; mirror common edge cases.
- Gate heavy crates via features where practical (progress, etc.).

## 4) Testing Strategy

- Unit tests comparing results with the `which` crate for a known corpus.

## 5) Acceptance Criteria

- No functional regressions; crate removed or made optional.
- No direct `which::which` usages remain outside the single implementation behind `Worker.which()`; no duplicate PATH search utilities are introduced.

## 6) References

- `src/system/worker/packages.rs`
- `PROJECT_PLANS/10_DEPENDENCY_FOOTPRINT_TRIM.md`
