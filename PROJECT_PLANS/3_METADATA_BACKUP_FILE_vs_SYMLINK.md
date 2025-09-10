# METADATA BACKUP: FILE vs SYMLINK

## 1) Scope (What this project changes)

- Explicit boundaries and non-goals.
  - Preserve full metadata for regular files on backup/restore; for symlinks, preserve linkness + target only.
  - Non-goal: introduce deep xattr/ACL management for symlinks without policy need.
- Source files/modules likely touched (paths + key symbols).
  - `src/symlink/ops.rs::{replace_file_with_symlink, restore_file}`
- User-visible behavior changes (if any).
  - None; internal backup fidelity increases for regular files; symlink metadata remains lean.

## 2) Rationale & Safety Objectives

- Why this is needed (short).
  - Ensure restorations maintain file semantics while avoiding overkill on symlink attributes that are not used here.
- Safety invariants.
  - Regular files: owner/mode/timestamps preserved.
  - Symlinks: link nature and destination preserved.
- Overkill → Lean replacement summary.
  - Replace deep symlink metadata preservation with a file/symlink split policy.

## 3) Architecture & Design

- High-level approach.
  - On backup (`fs::copy`), persist permissions and set timestamps (`utimensat`); on restore, atomic `renameat` as implemented. Symlinks are backed up as symlinks to the same target.
- Data model.
  - No persistent schema changes.
- Control flow: Staging → Validation → Commit → Verify → Rollback.
  - Staging copies and records metadata; commit swaps; verify compares kinds.
- Public interfaces.
  - None user-facing.

## 4) Failure Modes & Guarantees

- Failures: missing backups, permission errors; surfaced as `Error::RestoreBackupMissing` or `Error::Io`.
- Rollback: `restore_file` already atomic via `renameat`.
- Idempotency: repeat copy preserves attributes deterministically.
- Concurrency: no additional concerns.

## 5) Preflight & Post-Change Verification

- Preflight: verify backup path writable; ensure target not immutable.
- Post: confirm kind (symlink vs regular) and, for files, key metadata parity if feasible.

## 6) Observability & Audit

- Add duration and backup_path fields (already present in `AuditFields`).
- Optional: record preserved fields for files.

## 7) Security & Policy

- Ownership/labels/xattrs.
  - Preserve file uid/gid/mode/timestamps. Skip symlink deep metadata unless policy emerges.

## 8) Migration Plan

- No incompatible changes.

## 9) Testing Strategy

- Unit: backup preserves permissions bits.
- E2E: disable and verify restored targets are not symlinks (pattern in sudo-rs).

## 10) Acceptance Criteria (Must be true to ship)

- File backups restore owner/mode/timestamps.
- Symlink backups remain lean and correct.

## 11) Work Breakdown & Review Checklist

- Implement timestamp preservation; validate with stat comparisons.

## 12) References (Repo evidence only)

- `src/symlink/ops.rs::{replace_file_with_symlink, restore_file}`
- `src/experiments/sudors.rs::disable` (post-restore verification pattern)
- TODO_LIST_V2.md items: "Metadata backup & restore (file vs symlink split)", "Post-restore verifiers for non-sudo experiments"
