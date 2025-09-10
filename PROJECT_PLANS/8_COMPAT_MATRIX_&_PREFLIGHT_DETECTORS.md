# COMPATIBILITY MATRIX & PREFLIGHT DETECTORS

## 1) Scope (What this project changes)

- Explicit boundaries and non-goals.
  - Provide a small matrix of high-risk applets and known flag deltas; scan for risky usage and fail closed.
  - Non-goal: introduce a heavy shim layer at runtime.
- Source files/modules likely touched (paths + key symbols).
  - `src/experiments/*::{discover_applets}` (applets list for matrix)
  - `src/cli/{parser.rs, handler.rs}` (reporting)
  - `src/checks/compat.rs::{is_supported_distro}` (distro gating reference)
- User-visible behavior changes.
  - `oxidizr-arch check` outputs compatibility warnings/errors pre-commit.

## 2) Rationale & Safety Objectives

- Why.
  - Avoid risky substitutions for known-different flags/semantics.
- Safety invariants.
  - Fail-closed on detected risky flags for top applets.
- Overkill → Lean replacement summary.
  - Replace a giant runtime shim with a documented matrix + preflight detection + smoke tests.

## 3) Architecture & Design

- High-level approach.
  - Maintain a small in-repo matrix (JSON/TOML) of known risks for a short list of applets. Preflight scans installed scripts/services for these patterns and blocks commit if present without explicit override.
- Data model & structures.
  - `compat_matrix.json`: `{ applet: [{ pattern: "--flag", level: "warn|block" }, ...] }`.
- Control flow.
  - Preflight: scan → aggregate → emit report → block if any `block` level hit.
- Public interfaces.
  - CLI: `check` outputs the matrix results.

## 4) Failure Modes & Guarantees

- False positives: provide `--ignore-compat <applet:flag>` to override explicitly.

## 5) Preflight & Post-Change Verification

- Preflight: run detectors; show actionable advice.
- Post: smoke tests provide runtime signal.

## 6) Observability & Audit

- Log `compat_scan`, `hits=[…]`.

## 7) Security & Policy

- Prefer static scan; no runtime hooks beyond smoke tests.

## 8) Migration Plan

- Ship small seed matrix; expand conservatively.

## 9) Testing Strategy

- Unit: pattern matcher.
- E2E: known-risk script should block.

## 10) Acceptance Criteria (Must be true to ship)

- Matrix exists and detectors run by default in check/preflight.

## 11) Work Breakdown & Review Checklist

- Define matrix file → implement scanner → integrate with `check`.

## 12) References (Repo evidence only)

- `src/experiments/coreutils.rs::discover_applets`
- `src/experiments/findutils.rs::discover_applets`
- `src/checks/compat.rs::{is_supported_distro}`
- TODO_LIST_V2.md items: "Explicit experiment ordering gating", "Automatic smoke tests and rollback triggers"
