# Security Review Checklist
**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Threat model and hardening checklist for Switchyard library integrations.  
**Inputs reviewed:** SPEC §15 Security; PLAN/90-implementation-tiers.md; CODE: `src/fs/*`, `src/api/*`, `src/logging/*`, `src/policy/*`  
**Affected modules:** all

## Summary
- Primary risks are path traversal and TOCTOU races; mitigations in place: `SafePath`, `open_dir_nofollow`, `*at` syscalls, parent `fsync`.
- Secrets and volatile fields are masked in facts; redaction is enforced in dry-run.
- Locking, rescue profile verification, and auto-rollback reduce blast radius.

## Threat Model
- Inputs: paths (must be `SafePath`), environment (PATH, HOME), filesystem state.
- Adversary: Non-root user attempting to cause unsafe swaps or information leaks; hostile environment altering mounts; corrupted backup artifacts.

## Mitigations
- Path traversal: `fs/paths.rs::is_safe_path`, `types/SafePath`, parent `O_DIRECTORY|O_NOFOLLOW` handles.
- Atomicity: `renameat`, `symlinkat`, `unlinkat`, `fsync_parent_dir`.
- Cross-FS: explicit EXDEV handling and `allow_degraded_fs` policy.
- Backups: sidecars record prior topology; restore idempotence protects repeated actions.
- Observability: `redact_event` masks secrets; `FactsEmitter` isolates sinks.
- Locking: `FileLockManager` serializes mutations; policy enforces presence in Commit.

## Hardening Checklist
- Umask sane defaults (e.g., 022) when writing backups/sidecars.
- Sidecar integrity: consider signing sidecars or storing `bundle_hash` of backup payload.
- Validate `schema_version` across readers; support v2 with dual-read if adding fields.
- Limit environment influence: sanitize PATH for rescue checks in production.
- Drop privileges where possible in host processes integrating Switchyard.
- Enable read-only mounts enforcement in preflight (`extra_mount_checks`).

## Acceptance Criteria
- Documented checklist adopted in release process.
- Optional sidecar signing design drafted (future work).

## References
- SPEC: §15 Security Requirements; §2 Safety Model
- PLAN: 90-implementation-tiers.md
- CODE: `src/fs/atomic.rs`, `src/fs/backup.rs`, `src/fs/restore.rs`, `src/logging/redact.rs`

## Round 1 Peer Review (AI 2, 2025-09-12 15:06 +02:00)

**Claims Verified:**
- ✅ Path traversal mitigations: `SafePath` type exists in `src/types/safepath.rs`, `open_dir_nofollow` used in atomic operations
- ✅ Atomicity via `*at` syscalls: `renameat`, `symlinkat`, `unlinkat` used throughout `src/fs/` modules
- ✅ Backup sidecar schema: `BackupSidecar` struct in `src/fs/backup.rs:244-252` records topology
- ✅ Redaction: `src/logging/redact.rs` exists and masks sensitive fields
- ✅ Locking: `FileLockManager` serializes mutations as verified in previous documents

**Key Citations:**
- `src/types/safepath.rs`: SafePath type definition
- `src/fs/atomic.rs`: Uses `open_dir_nofollow` and `*at` syscalls
- `src/fs/backup.rs:244-252`: BackupSidecar schema with prior_kind/prior_dest
- `src/logging/redact.rs`: Event redaction implementation

**Summary of Edits:** All security claims are supported by the codebase. The threat model and mitigations accurately reflect the implemented security measures.

Reviewed and updated in Round 1 by AI 2 on 2025-09-12 15:06 +02:00
