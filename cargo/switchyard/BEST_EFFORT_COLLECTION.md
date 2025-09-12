# Best Effort in Switchyard

This document explains what “best effort” means in the Switchyard codebase, what its opposite is in programming terms, and catalogs all places that explicitly mention best‑effort behavior (or use it implicitly in comments). For each occurrence, it explains the related code and why the behavior is best‑effort.

## What “best effort” means

In programming, a best‑effort operation attempts to perform a task or collect a signal but does not offer a hard guarantee. Failures or ambiguity are treated as non‑fatal: the program proceeds with a conservative default, often recording a note for observability. Best‑effort is used when:

- External tools or platform features may be missing or unreliable
- Checks are heuristic and should not halt execution
- The result is advisory or for telemetry, not a gate
- Policy allows degraded behavior for resilience

### The opposite of best‑effort

The opposite is strict, fail‑closed enforcement with hard guarantees. In such code paths, failure is fatal (error/STOP), and the program refuses to proceed. In Switchyard, these are the normative, deterministic guarantees defined in SPEC and enforced via policy, e.g., atomicity requirements, backup presence when required, integrity verification when enabled, and locking in production. Think “MUST” semantics vs “TRY and continue.”

---

## Collected occurrences and explanations

Below are all occurrences that reference best‑effort (or clearly implement it), with citations.

### Preflight checks

- `src/preflight/checks.rs::check_suid_sgid_risk()`
  - Comment: “Best‑effort check for SUID/SGID risk … On errors reading metadata, returns Ok(false).”
  - Why best‑effort: If metadata cannot be read, we avoid false STOPs and instead return `Ok(false)` so policy can proceed or warn. Stops are reserved for deterministically known risks.

- `src/preflight/checks.rs::check_immutable()`
  - Comment: “Heuristic via lsattr -d; best‑effort and non‑fatal when unavailable … If lsattr is missing or fails, this returns Ok(()).”
  - Why best‑effort: `lsattr` may be absent in minimal containers. Immutability detection here is advisory; only explicit detection should STOP. This prevents flakiness across environments.

- `src/preflight.rs` (module docs)
  - “This module provides best‑effort filesystem and policy gating checks …”
  - Why best‑effort: Preflight aims to inform with conservative probes and only STOP on clear, policy‑required violations.

- `src/api/preflight/mod.rs` (provenance and summary chain)
  - “Provenance best‑effort” when enriching rows via `OwnershipOracle`.
  - “Best‑effort: co‑emit E_OWNERSHIP if any stop references ownership.”
  - Why best‑effort: Provenance enrichment and summary error chains are advisory signals for operators and analytics; they shouldn’t fail the stage if adapters are absent.

### Apply stage and summary mapping

- `src/api/apply/mod.rs::lock_backend_label()`
  - “Best‑effort dynamic type name” for lock backend labeling.
  - Why best‑effort: Used purely for observability; if we can’t map a known type, we emit a generic label without failing.

- `src/api/apply/mod.rs` (apply.summary)
  - “Attach perf aggregate (best‑effort, zeros in DryRun may be redacted).”
  - Why best‑effort: Timing signals may be zero or omitted (especially in DryRun) to preserve determinism and redaction rules. Not safety critical.

- `src/api/apply/mod.rs` (failure mapping)
  - “Compute chain best‑effort from collected error messages,” delegating to `api::errors::infer_summary_error_ids()`.
  - Why best‑effort: The mapping is heuristic for analytics and routing; it must not block or be treated as authoritative.

- `src/api/errors.rs::infer_summary_error_ids()`
  - “Best‑effort mapping from apply‑stage error strings to a chain of stable summary error IDs.”
  - Why best‑effort: String matching is inherently heuristic. We always include a top‑level `E_POLICY` but do not fail if specific matches aren’t found.

### Restore and backups

- `src/fs/restore.rs::restore_file()` and `restore_file_prev()`
  - Multiple comments:
    - “Without payload, either best‑effort noop or error.”
    - “Best‑effort: skip integrity‑enforced restore” (when policy allows) on payload hash mismatch.
  - Why best‑effort: `Policy.force_restore_best_effort` and `!require_sidecar_integrity` allow recovery to proceed without strict artifacts (e.g., missing payload, unverifiable hash), favoring availability over strict enforcement when configured.

- `src/policy/config.rs` (model fields)
  - `force_restore_best_effort`: “When true, `restore_file()` is allowed to succeed without a backup payload present (best‑effort).”
  - `require_sidecar_integrity`: “When false, integrity verification may be skipped and is treated as best‑effort.”
  - `preservation_tier`: “advisory; current implementation captures extended fields best‑effort.”
  - Why best‑effort: These flags explicitly toggle strict vs advisory behavior for restore and preservation fidelity.

- `src/fs/backup.rs::create_snapshot()`
  - “Durability: best‑effort parent fsync” after writing sidecars/symlink backups.
  - Why best‑effort: Fsync of the parent is attempted to improve durability, but errors are ignored to avoid turning observability/durability hints into hard failures; subsequent operations also sync where critical.

### Atomic swap and degraded mode

- `src/fs/atomic.rs::atomic_symlink_swap()`
  - “Best‑effort unlink temporary name if present (ignore errors).”
  - “Fall back: best‑effort non‑atomic replacement” when `EXDEV` and `allow_degraded=true`.
  - Why best‑effort: Cleaning a left‑over temp name should not fail the operation. Cross‑FS symlink replacement cannot be fully atomic; the fallback is intentionally degraded and policy‑guarded.

- `SPEC/SPEC_UPDATE_0002.md` and `SPEC/SPEC.md` (§2.10)
  - Clarify that EXDEV degraded symlink semantics are a “best‑effort degraded fallback.”
  - Why best‑effort: Filesystem semantics constrain atomicity here; we proceed only under explicit policy and record `degraded=true`.

- `README.md` (“Degraded Symlink Semantics”)
  - “uses unlink + `symlinkat` as a best‑effort degraded fallback”
  - Why best‑effort: Mirrors the SPEC and implementation details for operator awareness.

- `SPEC/features/atomic_swap.feature` and `SPEC/features/steps-contract.yaml`
  - Gherkin steps explicitly mention “best‑effort degraded fallback … when EXDEV occurs.”
  - Why best‑effort: Acceptance tests document and validate the degraded path under policy.

### Policy gating and inventory docs

- `DOCS/INVENTORY/SAFETY_Backup_and_Sidecar.md`
  - “Sidecar integrity is best‑effort unless `require_sidecar_integrity` is enforced in restore.”

- `DOCS/INVENTORY/SAFETY_Policy_Gating_and_Preflight.md`
  - “Some checks are best‑effort (immutability via `lsattr`), may be inconclusive.”

- `PLAN/60-rollback-exdev.md` and `PLAN/70-pseudocode.md`
  - Pseudocode notes rollback in reverse order as “best effort” when recording partial restoration on errors.
  - Why best‑effort: On rollback failures, we still collect and emit as much state as possible for operator recovery, without masking the failure.

### Filesystem metadata and mounts

- `src/fs/meta.rs::detect_preservation_capabilities()`
  - Comment: “xattrs … best‑effort probe.”
  - Why best‑effort: Capability detection varies by platform and permissions; the probe falls back to false on error.

- `src/fs/mount.rs::ProcStatfsInspector::parse_proc_mounts()`
  - “Canonicalize best‑effort; if it fails, still proceed with the raw path.”
  - Why best‑effort: Canonicalization can fail (permissions, broken links). We fall back to best match without halting mount checks entirely.

### Apply handlers

- `src/api/apply/handlers.rs::handle_restore()`
  - “Pre‑compute sidecar integrity verification (best‑effort) before restore” — computes `sidecar_integrity_verified` when possible.
  - Why best‑effort: The verification flag is advisory in facts; strict enforcement is governed by policy switches passed into `fs::restore_*` calls.

---

## Summary: When and why we choose best‑effort

- External/Non‑deterministic dependencies: e.g., `lsattr`, containerized environments.
- Analytics/Observability only: perf aggregates, error‑chain inference, provenance enrichment.
- Recovery‑first posture: allow restore to proceed when policy opts in, even if artifacts are partial.
- Filesystem constraints: degraded paths under EXDEV where atomicity isn’t achievable.
- Cross‑platform capability probes: xattrs, ownership, mounts.

Where safety matters, Switchyard defaults to strict, fail‑closed behavior (policy gates, atomic swaps, integrity when required). Best‑effort paths are deliberately scoped, observable, and controlled by policy.
