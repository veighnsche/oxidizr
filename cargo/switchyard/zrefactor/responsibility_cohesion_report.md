# Responsibility Cohesion Report — Files and Folders

This report lists where responsibilities are mixed in a single module, or spread too far apart, and proposes a concrete, “pristine” file/folder organization with acceptance checks.

## Executive Summary

- Centralize policy gating decisions under `policy::gating`.
- Keep `api/*` modules as thin stage orchestrators (no business rules, no JSON assembly).
- Split `fs/*` monoliths into cohesive submodules (backup/restore flow steps and helpers).
- Route all audit/logging via a single facade (StageLogger/EventBuilder) under `logging/`.
- Remove deprecated shims and public re-exports of low-level FS atoms.

---

## Findings and Recommendations

### 1) API orchestrators mixing concerns

- File: `src/api/apply/mod.rs`
  - Mixed concerns:
    - Orchestrates apply loop (OK).
    - Enforces locking policy and computes telemetry fields inline.
    - Performs plan-wide gating via `gating::gating_errors` (duplicates preflight; may drift from policy evaluator).
    - Builds attestation bundle JSON inline (lines ~417–444).
  - Recommendations:
    - Per-action gate via `policy::gating::evaluate_action(&policy, owner, &action)` before mutating; remove plan-wide `gating_errors`.
    - Extract attestation into `adapters::attest` or a helper returning a typed struct; stage only attaches it to fields.
    - Ensure all emissions go through `logging` facade (no JSON assembly here).

- File: `src/api/preflight/mod.rs`
  - Mixed concerns:
    - Implements policy checks inline: mount rw+exec, SUID/SGID, immutable, ownership, allow_roots/forbid_paths, extra_mount_checks.
    - Hard-coded `"/usr"` path for mount check in RestoreFromBackup branch.
    - Duplicate SUID/SGID check block (lines ~87–110 repeated).
  - Recommendations:
    - Thin orchestrator: call `policy::gating::evaluate_action(..)` per action and translate to rows.
    - Replace hard-coded `"/usr"` with iteration over `policy.extra_mount_checks`.
    - Remove duplicate SUID/SGID block.

- File: `src/api/plan.rs`
  - Concern:
    - Emits plan facts directly via `emit_plan_fact` (OK short-term) — migrate to StageLogger builder to avoid JSON literals.

- File: `src/api.rs`
  - Concern:
    - Uses `#[path]`-based includes.
  - Recommendation:
    - Convert to `src/api/mod.rs` with idiomatic `mod` declarations.

Acceptance checks (API)

- `grep -R "ensure_mount_rw_exec|check_suid_sgid_risk|check_immutable|allow_roots|forbid_paths" src/api` returns 0.
- `grep -R "gating_errors\(" src/api` returns 0.
- `grep -R "FactsEmitter::emit\(" src/api` returns 0; StageLogger used instead.

---

### 2) FS monoliths (backup/restore)

- Files: `src/fs/backup.rs` (~17k), `src/fs/restore.rs` (~30k)
  - Mixed concerns:
    - `backup.rs`: path generation, snapshot payload creation, sidecar read/write, index scanning, pruning logic.
    - `restore.rs`: integrity verification, idempotence, selector, steps and engine orchestration.
  - Recommendations: split by concern
    - `src/fs/backup/{mod.rs, snapshot.rs, sidecar.rs, index.rs, prune.rs}`
    - `src/fs/restore/{mod.rs, types.rs, integrity.rs, idempotence.rs, selector.rs, steps.rs, engine.rs}`
  - Acceptance:
    - Each submodule focused; files < ~800 LOC; top-level `fs/mod.rs` re-exports only high-level safe functions.

- File: `src/fs/mod.rs`
  - Concern:
    - Public re-exports of atoms: `atomic_symlink_swap`, `fsync_parent_dir`, `open_dir_nofollow`.
  - Recommendation:
    - Remove public re-exports; keep atoms internal (`pub(crate)`); expose high-level helpers only.
  - Acceptance:
    - `grep -R "open_dir_nofollow\|atomic_symlink_swap\|fsync_parent_dir" src/ | grep -v "src/fs/atomic"` returns 0.

---

### 3) Policy ownership of gating

- Files: `src/policy/gating.rs`, `src/api/preflight/mod.rs`, `src/api/apply/mod.rs`
  - Concern:
    - Rules implemented in stages; policy evaluator not the single source of truth.
  - Recommendation:
    - Implement `pub fn evaluate_action(policy: &Policy, owner: Option<&dyn OwnershipOracle>, act: &Action) -> ActionEvaluation` in `policy::gating`.
    - Stages call evaluator; delete duplicate logic from API files.
  - Acceptance:
    - `grep -R "evaluate_action\(" src/api` shows invocations only.

---

### 4) Logging and telemetry

- Files: `src/api/*`, `src/logging/audit.rs`
  - Concerns:
    - Stage modules assemble JSON fields directly via `emit_*` helpers; envelope/redaction spread.
  - Recommendation:
    - Introduce `StageLogger` / `EventBuilder` under `logging/audit.rs`; make legacy `emit_*` private → delete after migration.
  - Acceptance:
    - `grep -R "audit::emit_" src/api` returns 0.

---

### 5) Deprecated shims and leaky facades

- File: `src/lib.rs`
  - Concern: deprecated `pub use policy::rescue` top-level alias.
  - Action: remove; consumers import `switchyard::policy::rescue`.

- File: `src/adapters/mod.rs`
  - Concern: deprecated `adapters::lock_file::*` shim.
  - Action: remove; ensure curated re-exports remain for `FileLockManager` and friends.

Acceptance checks (deprecated)

- `grep -R "use switchyard::rescue" -n` returns 0.
- `grep -R "adapters::lock_file::" -n` returns 0.

---

## Proposed Pristine Layout

```
src/
  api/
    mod.rs
    plan.rs
    preflight/
      mod.rs
      rows.rs
    apply/
      mod.rs
      handlers.rs
    rollback.rs
  fs/
    mod.rs
    atomic.rs
    mount.rs
    paths.rs
    backup/
      mod.rs
      snapshot.rs
      sidecar.rs
      index.rs
      prune.rs
    restore/
      mod.rs
      types.rs
      integrity.rs
      idempotence.rs
      selector.rs
      steps.rs
      engine.rs
  logging/
    mod.rs
    facts.rs
    redact.rs
    audit.rs  (StageLogger/EventBuilder)
  policy/
    mod.rs
    types.rs
    profiles.rs
    gating.rs  (evaluate_action)
  types/
    ids.rs
    plan.rs
    report.rs
    safepath.rs
    error_id.rs
  lib.rs
```

---

## Migration Guardrails

- API thinness: no business rules in `src/api/**`.
- Policy evaluator is sole owner of gating.
- Logging facade centralizes envelope/redaction; no direct `FactsEmitter::emit` in API.
- FS atoms are not public; public facade offers safe functions only.

Greps to enforce

- `rg -n "evaluate_action\(" src/api` → calls only.
- `rg -n "FactsEmitter::emit\(" src/api` → 0.
- `rg -n "ensure_mount_rw_exec|check_suid_sgid_risk|check_immutable|allow_roots|forbid_paths" src/api` → 0.
- `rg -n "open_dir_nofollow|atomic_symlink_swap|fsync_parent_dir" src/ | rg -v "src/fs/atomic"` → 0.

---

## Notes and Small Fixes

- `src/api/preflight/mod.rs`: remove duplicate SUID/SGID block; replace `/usr` hard-code with policy.extra_mount_checks.
- `src/api/apply/mod.rs`: extract attestation bundle build to adapters/helper; orchestrator attaches typed result to fields only.
- `src/api.rs`: convert to `src/api/mod.rs` (no `#[path]`).

---

## Acceptance for this reorg report

- Acknowledged structure and checks added to `zrefactor/` INSTRUCTIONS.
- When executed, CI greps above pass and the public API surface remains focused and curated.
