# DEPENDENCY FOOTPRINT TRIM

## 1) Scope (What this project changes)

- Explicit boundaries and non-goals.
  - Replace `which` crate usage with a small internal PATH search; consider feature-gating heavier deps.
  - Non-goal: rewrite logging stack.
- Source files/modules likely touched (paths + key symbols).
  - `src/system/worker/packages.rs` (uses `which`)
  - `src/experiments/*::{discover_applets}` (PATH fallback)
  - `Cargo.toml` (features; not modified in this pass)
- User-visible behavior changes.
  - None.

## 2) Rationale & Safety Objectives

- Why.
  - Reduce trusted code base size.
- Safety invariants.
  - Equivalent behavior for PATH discovery.

## 3) Architecture & Design

- High-level approach.
  - Introduce a tiny `path_search` module to scan PATH for executables. Gate largish crates (e.g., progress bar) behind features.

## 4) Failure Modes & Guarantees

- PATH parsing edge cases: quote and empty segments handled properly.

## 5) Preflight & Post-Change Verification

- Unit: compare results vs current `which` for known cases.

## 6) Observability & Audit

- None.

## 7) Security & Policy

- N/A.

## 8) Migration Plan

- Keep both paths behind a feature; migrate callers.

## 9) Testing Strategy

- Unit tests for search.

## 10) Acceptance Criteria (Must be true to ship)

- No regressions in discovery; crate removed or made optional.

## 11) Work Breakdown & Review Checklist

- Add module → swap call sites → feature gate → remove crate.

## 12) References (Repo evidence only)

- `src/system/worker/packages.rs` (calls to `which`)
- `src/experiments/coreutils.rs::discover_applets`
- `src/experiments/findutils.rs::discover_applets`
- TODO_LIST_V2.md items: "Dependencies review", "Log exact per-applet path when falling back to PATH"
