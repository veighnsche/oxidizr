# Stream A — Profiles & Atomic Flip + Canary + Smoke Tests + Backup Semantics

## 1) Scope

- Replace N-per-target link commits with a single profile pointer flip.
- Provide GNU escape path and `canary --shell` using GNU profile.
- Run minimal smoke tests post-flip; auto-rollback on failure.
- Clarify backup semantics: files preserve owner/mode/timestamps; symlinks preserve linkness + target.

Touched modules (validated):

- `src/symlink/ops.rs::{atomic_symlink_swap, restore_file, backup_path}`
- `src/experiments/util.rs::{create_symlinks, restore_targets, resolve_usrbin}`
- `src/experiments/{coreutils.rs, findutils.rs}` (wire to build profile trees)
- `src/system/lock.rs::acquire`
- `src/state/mod.rs::{set_enabled, write_state_report}`
- New: profile layout helpers under `src/experiments/util.rs` or a new `src/profiles/mod.rs`
- CLI: new subcommand `canary --shell`; optional `profile --set {gnu|uutils}`

## Reuse existing infrastructure

To keep the system lean, implement Stream A by extending existing modules rather than creating parallel layers:

- Symlink/backup/restore must use `src/symlink/ops.rs::{replace_file_with_symlink, restore_file, atomic_symlink_swap}`. Do not add a second symlink implementation.
- The pointer flip should be implemented as an atomic `renameat` helper added to `src/symlink/ops.rs` and reused by experiments. No ad-hoc file ops elsewhere.
- Build/update of profile trees should reuse `src/experiments/util.rs::create_symlinks`-style staging, targeting profile dirs instead of `/usr/bin`.
- Post-commit smoke tests and rollback wiring should live in `src/experiments/*` and call existing restore helpers.
- All PATH lookups must go through `Worker.which()` (in `src/system/worker/fs_ops.rs`).
- All logging must go through `src/logging/{audit.rs, init.rs}` with `audit_event_fields`.
- Reuse `src/system/lock.rs::acquire`, `src/state/mod.rs`, and `src/ui/progress.rs` for locking, persistence, and progress.

Acceptance (reuse):

- No new symlink/backup/restore implementation is introduced; only `src/symlink/ops.rs` is extended.
- Pointer flip uses a helper in `src/symlink/ops.rs` that performs `renameat` + fsync, not ad‑hoc code in experiments.
- Experiments call shared helpers from `src/experiments/util.rs`; logs flow via `audit_event_fields` only.

## 2) Rationale & Safety Objectives

- Atomicity: one `renameat(2)` to flip `.../active` under `/usr/lib/oxidizr-arch/`.
- Rollback: single pointer re-flip; emergency per-target restore remains.
- Minimal surface: fewer moving parts at commit time.
- Auditability: single op with clear `from_profile` → `to_profile` fields (extend `AuditFields`).

## 3) Architecture & Design

- Layout: `/usr/lib/oxidizr-arch/profiles/{gnu,uutils}/bin`, and `/usr/lib/oxidizr-arch/active -> profiles/<current>`.
- `/usr/bin/<applet>` remains a stable symlink to `.../active/bin/<applet>`.
- Experiments still compute applets; commit becomes pointer switch.
- Implement helper to build/update profile trees using current `create_symlinks()` staging but targeting profiles, not `/usr/bin`.
- Add post-commit smoke runner (`ls|cp|find|xargs --version`) with rollback hook.

## 4) Failure Modes & Guarantees

- Missing profile tree → abort before flip.
- Permissions/immutable flags surfaced via existing checks; keep `restore_file()` as emergency fallback.
- Idempotency: flipping to same profile is a no-op.
- Concurrency: enforce `ProcessLock`.

## 5) Preflight & Post-Change Verification

- Preflight: ensure trees exist; target mount is `rw,exec`; ownership checks optional.
- Post: smoke tests; on failure, pointer re-flip and/or `restore_targets()`.

## 6) Observability & Audit

- Extend `AuditFields` with `from_profile`, `to_profile` (or use `artifacts` list) for `event=profile_flip`.
- Keep per-applet link logs during tree build (staging), but commit logs should be one flip event.

## 7) Security & Policy

- Environment sanitization: `LC_ALL=C` when probing binaries.
- Supply-chain policy is out-of-scope here (see Stream D).
- Safety decisions integration (see `PROJECT_PLANS/SAFETY_DECISIONS_AUDIT.md`):
  - Preserve `uid/gid/mode` and timestamps for regular files touched during backups/restores; symlinks preserve linkness + target only.
  - If a managed executable has `security.capability`, record before and ensure it is present after flip (post-flip validation). Implement via `system/security.rs::{get_capabilities,set_capabilities}`.
  - If SELinux/AppArmor labels are active, attempt a relabel on the active profile tree post-flip (best-effort), with structured audit of outcome.
  - ACLs: detect and warn pre/post; preserve only when `--preserve-acl` is set (policy gate).

## 8) Migration Plan

1. Introduce profile directory scaffolding and helper functions.
2. Populate GNU and uutils trees during enable paths (no `/usr/bin` writes).
3. Switch `/usr/bin/*` to point to `.../active/bin/*` and initialize `active` accordingly.
4. Add `canary --shell` subcommand to prepend GNU profile to PATH.
5. Add smoke test runner and rollback glue.

## 9) Testing Strategy

- Unit: atomic flip (renameat) invariants and parent fsync.
- Integration: enable → flip → smoke → revert; fault injection: make one applet fail to trigger rollback.
- E2E: run in Docker via `test-orch/` across Arch derivatives.

## 10) Acceptance Criteria

- Pointer flip replaces per-target commit; flips are atomic and reversible.
- Smoke failures roll back automatically and leave system healthy.
- If `security.capability` existed on managed executables pre-flip, it is preserved post-flip.
- If SELinux/AppArmor labels are active, a relabel attempt on the active profile tree is performed and audited.
- ACLs are detected and warnings emitted with remediation unless `--preserve-acl` is set.

## 11) Work Breakdown

- Profile scaffolding + helpers.
- Tree population wiring in experiments.
- Flip commit + verification + rollback path.
- Canary shell CLI.
- Backup semantics adjustments where needed in `src/symlink/ops.rs` (file vs symlink split retained).

## 12) References

- `src/symlink/ops.rs::{atomic_symlink_swap, restore_file}`
- `src/experiments/util.rs::{create_symlinks, resolve_usrbin}`
- `src/experiments/{coreutils.rs, findutils.rs}`
- `src/system/lock.rs::acquire`
- `PROJECT_PLANS/1_PROFILES_&_ATOMIC_FLIP.md`, `6_CANARY_...md`, `7_SMOKE_TESTS_...md`, `3_METADATA_BACKUP_...md`
