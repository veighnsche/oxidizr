# Error Taxonomy & Exit-code Mapping

**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Inventory of `ErrorId` and exit-code mappings, where they are emitted (apply, preflight, gating, smoke), overlaps, and a unified guidance table for downstream CLIs.  
**Inputs reviewed:** SPEC §6 (Error Taxonomy), SPEC `error_codes.toml`, CODE `src/api/errors.rs`, `src/api/apply/*`, `src/api/preflight/mod.rs`, `src/policy/gating.rs`  
**Affected modules:** `src/api/errors.rs`, `src/api/apply/mod.rs`, `src/api/apply/handlers.rs`, `src/api/preflight/mod.rs`, `src/policy/gating.rs`

## Summary

- `ErrorId` is centralized in `src/api/errors.rs` and aligned with SPEC `error_codes.toml`. Mappings are consistent across preflight and apply.
- Emission points are explicit: locking and gating in `apply/mod.rs`, swap/EXDEV in `apply/handlers.rs`, restore in `apply/handlers.rs`, preflight STOP summary in `preflight/mod.rs`.
- Generic non-mapped errors default to `E_GENERIC` but are rarely emitted directly; we recommend confining to unexpected conditions only.

## Inventory / Findings

- Error identifiers and mapping (`src/api/errors.rs`):
  - `E_POLICY` → exit 10
  - `E_OWNERSHIP` → exit 20
  - `E_LOCKING` → exit 30
  - `E_ATOMIC_SWAP` → exit 40
  - `E_EXDEV` → exit 50
  - `E_BACKUP_MISSING` → exit 60
  - `E_RESTORE_FAILED` → exit 70
  - `E_SMOKE` → exit 80
  - `E_GENERIC` → exit 1

- Emission sites (non-exhaustive but representative):
  - Locking: `src/api/apply/mod.rs`
    - On lock acquisition failure or missing lock manager when required, emits `apply.attempt` failure and summary `apply.result` with `E_LOCKING` (30).
  - Policy gating: `src/api/apply/mod.rs` and `src/policy/gating.rs`
    - When `override_preflight=false` and plan violates gates, per-action `apply.result` rows carry `E_POLICY` (10) and a summary is emitted with `E_POLICY`.
  - Atomic swap and EXDEV: `src/api/apply/handlers.rs::handle_ensure_symlink`
    - On `EXDEV` with policy disallowing degraded fallback: `E_EXDEV` (50), plus `degraded=false`, `degraded_reason="exdev_fallback"`, `error_detail="exdev_fallback_failed"` guidance.
    - Other IO failures during swap: `E_ATOMIC_SWAP` (40).
  - Restore path: `src/api/apply/handlers.rs::handle_restore`
    - `NotFound` on required payload: `E_BACKUP_MISSING` (60); other errors → `E_RESTORE_FAILED` (70).
  - Preflight summary: `src/api/preflight/mod.rs`
    - Any STOP conditions produce a summary with `E_POLICY` (10) and `exit_code=10`.
  - Smoke tests: `src/api/apply/mod.rs`
    - Missing runner when required or failing suite yields summary `E_SMOKE` (80).

- Overlaps/ambiguities
  - Policy vs ownership: Ownership errors are currently surfaced as human strings in preflight and could be promoted to `E_OWNERSHIP` in facts where appropriate.
  - Summary error_id default: For non-smoke failures, summary uses `E_POLICY` by default even when the underlying cause is `E_ATOMIC_SWAP` or `E_EXDEV`. Consider surfacing the “dominant” error_id at summary level or including `error_ids: []` array.

## Recommendations

1. Promote strict ownership failures to emit `error_id=E_OWNERSHIP` in preflight rows (and summary when sole STOP reason), with `exit_code=20`. Touch: `src/api/preflight/mod.rs` and `src/policy/gating.rs`.
2. Summary mapping policy: In `apply/mod.rs`, when any per-action emitted `error_id` exists, set summary `error_id` to the first/most severe rather than defaulting to `E_POLICY`. Alternatively, add `summary_error_ids: [..]` field. Keep exit code aligned to primary.
3. Add `E_PRECHECK` reserved range (90s) only if future non-policy prechecks emerge, otherwise keep the current compact mapping.
4. Document the error mapping in Rustdoc on `ErrorId` and link to SPEC `error_codes.toml`. Enforce via a unit test comparing `exit_code_for` to the TOML.

## Risks & Trade-offs

- Changing summary `error_id` may affect downstream consumers; mitigate with additive `summary_error_ids` or a minor version bump.

## Spec/Docs deltas

- SPEC §6: Clarify that apply summary should reflect the dominant cause or list all causes. Add example.

## Acceptance Criteria

- Tests confirm emission sites map to the expected `ErrorId`/exit codes.
- Preflight ownership STOP emits `E_OWNERSHIP` where applicable.
- Apply summary provides a representative `error_id` without losing fidelity.

## References

- SPEC: §6 Error Taxonomy; `SPEC/error_codes.toml`
- PLAN: 30-errors-and-exit-codes.md; 12-api-module.md
- CODE: `src/api/errors.rs`, `src/api/apply/mod.rs`, `src/api/apply/handlers.rs`, `src/api/preflight/mod.rs`, `src/policy/gating.rs`

## Round 1 Peer Review (AI 1, 2025-09-12 15:14 +02:00)

- Claims verified
  - Centralized mapping of `ErrorId` to exit codes aligns with SPEC.
    - Proof: `src/api/errors.rs::exit_code_for()` maps IDs to codes (lines 61–73); `SPEC/error_codes.toml` lists the same values (lines 1–11).
  - Emission sites
    - Locking: `src/api/apply/mod.rs` emits `E_LOCKING` on lock acquisition failure (lines 66–87) and when missing lock manager in Commit (lines 101–131); summary `apply.result` includes `E_LOCKING` and `exit_code=30` (lines 114–121).
    - Policy gating: `src/api/apply/mod.rs` maps gating failures to per-action `apply.result` with `E_POLICY` and a summary with `E_POLICY`/10 (lines 160–202, esp. 167–183 and 185–193).
    - Atomic swap / EXDEV: `src/api/apply/handlers.rs::handle_ensure_symlink()` maps EXDEV to `E_EXDEV` and other IO failures to `E_ATOMIC_SWAP` (lines 61–70), emitting per-action failure with `error_id` and `exit_code` (lines 91–95) and setting `degraded=false`, `degraded_reason` and `error_detail` for EXDEV (lines 81–85).
    - Restore: `src/api/apply/handlers.rs::handle_restore()` maps `NotFound` to `E_BACKUP_MISSING` and others to `E_RESTORE_FAILED` (lines 191–209), emitting per-action failure with id/code (lines 206–208).
    - Preflight summary STOP: `src/api/preflight/mod.rs::run()` emits summary `error_id=E_POLICY` and `exit_code=10` when stops exist (lines 255–270).
  - Summary mapping default behavior
    - Proof: `src/api/apply/mod.rs` sets summary `error_id` to `E_SMOKE` only if smoke fails (lines 390–399); otherwise defaults to `E_POLICY` (lines 401–406).

- Key citations
  - `src/api/errors.rs::{ErrorId, id_str, exit_code_for}`
  - `src/api/apply/mod.rs::{run}`
  - `src/api/apply/handlers.rs::{handle_ensure_symlink, handle_restore}`
  - `src/api/preflight/mod.rs::run`
  - `SPEC/error_codes.toml`

- Summary of edits
  - Added precise code/spec citations confirming mappings and emission sites; clarified default summary mapping behavior. Recommendations retained (consider `summary_error_ids`).

Reviewed and updated in Round 1 by AI 1 on 2025-09-12 15:14 +02:00
