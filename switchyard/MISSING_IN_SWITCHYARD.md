# Switchyard — Missing Safety Features

This document lists safety and infrastructure features present in `oxidizr-arch` (docs and code under `src/`) that are not yet implemented or fully integrated in `switchyard/src/`. The aim is to guide extraction to make Switchyard reusable across distros and experiment bundles.

Sources reviewed:

- Requirements: `AUDIT_CHECKLIST.md`, `SAFETY_MEASURES.md`, `PROJECT_PLANS/*.md`
- Product code: `src/logging/*`, `src/system/*`, `src/symlink/ops.rs`, `src/experiments/util.rs`, `src/state/mod.rs`, `src/cli/*`
- Switchyard code: `switchyard/src/{api.rs,preflight.rs,symlink.rs,fs_ops.rs}`

Current Switchyard scope (baseline):

- Atomic symlink swap with per-target backup/restore, parent `fsync`, `O_NOFOLLOW` safety.
- Preflight checks: mount `rw` and not `noexec`, immutable bit (`lsattr`), source trust (root-owned, not world-writable, not under `$HOME` unless forced), basic path traversal defense, allow/forbid path policy gates.
- Simple planner (`PlanInput` → `Plan`) and executor with dry-run support.
- Hooks for `FactsEmitter`/`AuditSink` exist but are not used in code paths.

---

## Gaps vs. Safety Requirements and Product Code

- __Preflight diff and plan rendering__
  - Missing: Structured preflight "diff" of current vs planned state per target (kind/dest before→after, provenance, policy annotations) as described in `PROJECT_PLANS/B_PREFLIGHT_PLAN_COMPAT_UX.md` and referenced in `SAFETY_MEASURES.md`.
  - Today: `Switchyard::preflight()` only validates safety; it does not compute/report a per-target plan row.

- __Ownership and provenance checks__
  - Missing: Package ownership verification used in product (`src/system/worker/packages.rs::verify_owner_for_target`, `query_file_owner`) and policy `strict_ownership` enforcement.
  - Note: `Policy.strict_ownership` exists in Switchyard but is not enforced anywhere.

- __Audit integration and observability__
  - Missing: Wiring of `FactsEmitter`/`AuditSink` into `plan()`, `preflight()`, and `apply()` to emit machine-readable events and human logs.
  - Missing: Per-operation audit bundles/fields (duration per action is tracked but not emitted; no op_id, no artifacts listing, etc.). See `src/logging/audit.rs::{audit_event_fields,audit_op}`.

- __Attestation, hashing, signatures, SBOM__
  - Missing: Selective hashing of before/after states, per-op JSONL artifact with optional detached signature, and a verifier CLI per `PROJECT_PLANS/C_AUDIT_ATTESTATION_AND_DOCS.md` and `AUDIT_CHECKLIST.md` (cryptographic hashes, provenance completeness).

- __Health verification and rollback automation__
  - Missing: Post-commit smoke tests and automatic rollback triggers when tests fail, as required by `AUDIT_CHECKLIST.md` and `PROJECT_PLANS/A_PROFILES_FLIP_CANARY_SMOKES_BACKUP.md`.
  - Missing: Library-level rollback orchestration across a multi-action plan (apply currently executes best-effort with no transactional rollback across actions).

- __Profiles and atomic pointer flip__
  - Missing: Profile tree helpers and an atomic pointer flip (`renameat` + parent `fsync`) to swap `.../active` profile, shifting from N-per-target commits to a single atomic flip. See `PROJECT_PLANS/A_PROFILES_FLIP_CANARY_SMOKES_BACKUP.md`.

- __Compatibility matrix and policy scanners__
  - Missing: Compat matrix ingestion and scanners to surface risky flags/patterns during preflight (capabilities/ACL warnings, provenance). See `PROJECT_PLANS/B_PREFLIGHT_PLAN_COMPAT_UX.md`.

- __Extended metadata preservation and checks__
  - Missing: xattrs/ACLs/capabilities detection and preservation policies described in `SAFETY_MEASURES.md` and `PROJECT_PLANS/A_PROFILES_FLIP_CANARY_SMOKES_BACKUP.md`.
  - Current backups preserve permissions for regular files only; no capability/ACL/xattr handling.

- __Process-level locking__
  - Missing: Single-instance lock (`/run/lock/oxidizr-arch.lock`) used in product (`src/system/lock.rs`). Switchyard provides no locking primitive to prevent concurrent mutating operations.

- __State management and reporting__
  - Missing: Idempotent state persistence and human-readable state reports used by product (`src/state/mod.rs::{save_state, write_state_report}`) to support recovery and observability.

- __Supply chain and package operations policy__
  - Missing: Signature/checksum verification of binaries, provenance capture (owner, repo presence), SBOM-lite emission per `AUDIT_CHECKLIST.md` and `PROJECT_PLANS/C_AUDIT_ATTESTATION_AND_DOCS.md`.
  - Note: Product policies in `src/system/worker/packages.rs` (repo gating, lock wait, AUR fallback rules) are not represented in Switchyard; consider defining abstract traits so distro-specific workers can plug into preflight.

- __Determinism and dry-run posture__
  - Missing: Library-level guarantees and testing hooks for deterministic outputs (no env/locale/time sensitivity) as noted in `AUDIT_CHECKLIST.md`.
  - `ApplyMode::default()` is `DryRun`, but preflight/apply do not record deterministic plan rows for verification.

- __Human UX and verbosity contract__
  - Missing: Stable human logging levels/formatting contract (`src/logging/init.rs::HumanFormatter`) and dry-run gating for sinks.
  - Library traits exist but lack a default adapter that mirrors the product’s behavior.

- __Rescue toolset presence checks__
  - Missing: Preflight checks for fallback tool availability (busybox/static core) suggested by `AUDIT_CHECKLIST.md`.

- __Initramfs/emergency shell access__
  - Missing: Hooks or checks ensuring essential commands are available in rescue contexts (documented gap in `AUDIT_CHECKLIST.md`).

---

## Suggested Extraction Targets (from product → Switchyard)

- __Ownership/Provenance Trait__
  - Define a `ProvenanceProvider` trait (query owner, repo presence, verify strict ownership) and invoke it from `preflight()` when `Policy.strict_ownership` is true.
  - Source: `src/system/worker/packages.rs::{query_file_owner, verify_owner_for_target, repo_has_package}`.

- __Audit Hooks__
  - Call `FactsEmitter::emit` and `AuditSink::log` from `plan()`, `preflight()`, and `apply()` with structured fields: action, source, target, duration_ms, decision, errors.
  - Optionally define an op buffer interface to facilitate attestation.

- __Preflight Diff Model__
  - Introduce `PreflightItem { target, current_kind, current_dest?, planned_kind, planned_dest?, policy_ok, provenance }` and return it in a richer `PreflightReport`.
  - Source patterns: `PROJECT_PLANS/B_PREFLIGHT_PLAN_COMPAT_UX.md`.

- __Atomic Profile Flip__
  - Provide `profiles` module with `rename_active_pointer(active: &Path, new_target: &Path)` (parent `O_NOFOLLOW` + `renameat` + fsync) and simple tree validators.

- __Metadata Preservation__
  - Optional helpers to read/apply capabilities and ACLs or to detect and warn (policy-driven). See plan references.

- __Locking and State (thin adapters)__
  - Provide optional primitives or traits that can be implemented by the host application; Switchyard preflight/apply should be able to request a lock and optionally write state artifacts via an adapter.

- __Health Checks Interface__
  - Define a pluggable `SmokeTestRunner` trait; integrate post-apply with rollback on failure.

- __Attestation Interface__
  - Expose an optional `Attestor` trait for hashing/signing of per-operation artifacts and integrate it into `apply()` lifecycle.

---

## Quick Mapping of Product Code Not Yet Reflected in Switchyard

- `src/system/fs_checks.rs` — wraps Switchyard preflight checks; OK.
- `src/system/worker/packages.rs` — provenance/ownership, repo checks, lock waits, AUR policy — __missing__ in Switchyard (should be abstracted via traits).
- `src/logging/{audit.rs,init.rs}` — structured JSONL sink and human logs — __missing integration__ (traits exist but unused).
- `src/state/mod.rs` — persistence/reporting — __missing__.
- `src/system/lock.rs` — process lock — __missing__.
- `src/experiments/util.rs::{create_symlinks, restore_targets}` — product orchestration; Switchyard covers only the low-level swap/restore. Preflight diff and smoke tests — __missing__.
- `PROJECT_PLANS/*` — profiles, compat matrix, attestation — __missing__ modules and hooks.

---

## Priority Recommendations

1. Wire `FactsEmitter`/`AuditSink` throughout `plan/preflight/apply` and surface durations/errors.
2. Enforce `Policy.strict_ownership` via a pluggable provenance trait and add to preflight stops/warnings.
3. Add preflight diff model and richer `PreflightReport` rows.
4. Provide atomic profile flip helper and thin profile validation utilities.
5. Define interfaces for smoke tests and attestation; integrate with apply lifecycle and rollback hooks.
6. Add optional locking/state adapters so host apps can reuse Switchyard’s transaction boundary semantics.
