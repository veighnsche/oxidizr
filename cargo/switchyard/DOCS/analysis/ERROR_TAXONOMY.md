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

## Round 2 Gap Analysis (AI 4, 2025-09-12 15:38 CET)

- **Invariant: Error reporting provides detailed and actionable information for all failure scenarios.**
  - **Assumption (from doc):** The document assumes that error reporting, especially in summaries, should reflect the specific causes of failures to aid CLI consumers in diagnosing issues, recommending the addition of `summary_error_ids` to capture multiple error causes (`ERROR_TAXONOMY.md:45`, `ERROR_TAXONOMY.md:50-52`).
  - **Reality (evidence):** In the current implementation, apply and preflight summaries often default to a single generic `error_id` like `E_POLICY` even when multiple or specific errors occur (`src/api/apply/mod.rs:401-406`, `src/api/preflight/mod.rs:255-270`). There is no mechanism like `summary_error_ids` to list all contributing error IDs.
  - **Gap:** Collapsing multiple error conditions into a single generic `error_id` in summaries limits the diagnostic information available to CLI consumers, violating the expectation of detailed and actionable error reporting for complex failure scenarios.
  - **Mitigations:** Implement the recommended `summary_error_ids` array in apply and preflight summaries to capture all unique error IDs encountered (`src/api/apply/mod.rs`, `src/api/preflight/mod.rs`). Alternatively, prioritize the most severe or specific `error_id` for the summary while logging details in per-action facts. Update `SPEC/error_codes.toml` and `SPEC §6` to document this behavior.
  - **Impacted users:** CLI integrators and end-users who need precise error information to troubleshoot and resolve issues, especially in multi-step operations with multiple failure points.
  - **Follow-ups:** Flag this as a medium-severity usability gap for Round 3. Plan to enhance summary error reporting with detailed error arrays in Round 4.

- **Invariant: Error handling for ownership issues is distinctly identified for user action.**
  - **Assumption (from doc):** The document assumes that strict ownership failures should be distinctly reported as `E_OWNERSHIP` with a specific exit code to guide users on permission-related issues (`ERROR_TAXONOMY.md:19`, `ERROR_TAXONOMY.md:49-50`).
  - **Reality (evidence):** While `E_OWNERSHIP` is defined with exit code 20 in `src/api/errors.rs:61-73`, the current preflight implementation often reports ownership issues as generic human-readable strings in `notes` without setting `error_id` to `E_OWNERSHIP` (`src/api/preflight/mod.rs`, `src/policy/gating.rs`). These are then summarized under `E_POLICY`.
  - **Gap:** Ownership-related errors are not consistently tagged with `E_OWNERSHIP`, reducing the clarity for CLI consumers who expect a specific error code to automate or guide permission fixes. This violates the expectation of distinct error categorization for actionable response.
  - **Mitigations:** Update preflight and gating logic to emit `error_id=E_OWNERSHIP` for ownership-specific failures in per-action rows and ensure this is reflected in summaries when it is the primary cause (`src/api/preflight/mod.rs`, `src/policy/gating.rs`). Add test cases to verify consistent error ID usage for ownership issues.
  - **Impacted users:** System administrators and CLI users who encounter permission or ownership issues and need clear, specific error codes to address them efficiently.
  - **Follow-ups:** Flag this as a low-to-medium severity usability gap for Round 3. Plan to implement distinct ownership error tagging in Round 4.

Gap analysis in Round 2 by AI 4 on 2025-09-12 15:38 CET

## Round 3 Severity Assessment (AI 3, 2025-09-12 15:49+02:00)

- **Title:** Error summaries lose detail by collapsing multiple causes into one ID
- **Category:** Observability (DX/Usability)
- **Impact:** 3  **Likelihood:** 4  **Confidence:** 5  → **Priority:** 2  **Severity:** S3
- **Disposition:** Implement  **LHF:** Yes
- **Feasibility:** Medium  **Complexity:** 3
- **Why update vs why not:** This is the same issue identified in the `OBSERVABILITY_FACTS_SCHEMA.md` analysis. Collapsing multiple errors into a generic `E_POLICY` significantly harms debuggability. Implementing a `summary_error_ids` array provides crucial diagnostic information with low risk.
- **Evidence:** `src/api/apply/mod.rs:401-406` and `src/api/preflight/mod.rs:255-270` show the default fallback to `E_POLICY` in summaries.
- **Next step:** Implement the `summary_error_ids` array in preflight and apply summary facts during Round 4.

- **Title:** Ownership errors are not distinctly identified with E_OWNERSHIP
- **Category:** Observability (DX/Usability)
- **Impact:** 2  **Likelihood:** 4  **Confidence:** 5  → **Priority:** 2  **Severity:** S3
- **Disposition:** Implement  **LHF:** Yes
- **Feasibility:** High  **Complexity:** 2
- **Why update vs why not:** Failing to use a specific error code for a common class of failures (ownership) makes it harder for users and automation to react appropriately. Correctly tagging these errors is a low-effort change that improves the clarity and actionability of the tool's output.
- **Evidence:** Preflight checks for ownership currently result in human-readable notes rather than a machine-readable `E_OWNERSHIP` error ID, as noted in the Round 2 analysis.
- **Next step:** Update the preflight and gating logic in `src/policy/gating.rs` to emit `E_OWNERSHIP` for relevant failures in Round 4.

Severity assessed in Round 3 by AI 3 on 2025-09-12 15:49+02:00
