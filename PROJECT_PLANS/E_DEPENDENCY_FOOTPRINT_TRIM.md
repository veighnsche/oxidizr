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
