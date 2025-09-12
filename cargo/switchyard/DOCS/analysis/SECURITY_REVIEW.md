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

## Round 2 Gap Analysis (AI 1, 2025-09-12 15:22 +02:00)

- Invariant: Sidecar integrity and rollback trust
  - Assumption (from doc): Backups/sidecars are reliable for recovery.
  - Reality (evidence): `src/fs/backup.rs::write_sidecar()` uses path-based `File::create` and does not `fsync` the parent directory; symlink backups use `std::os::unix::fs::symlink` (path-based) (backup.rs lines 137–151, 262–270). No signature or cryptographic binding between payload and sidecar. Apply facts include attestation for plan bundles (not sidecars) in `src/api/apply/mod.rs` (lines 359–384).
  - Gap: Crash after write may lose sidecar or payload; integrity not verifiable. Operators cannot detect tampering or partial writes.
  - Mitigations: Move to `open_dir_nofollow(parent)` + `openat`/`symlinkat` and `fsync_parent_dir(backup)` for durability; add optional sidecar signing (e.g., include `bundle_hash` of payload in sidecar and sign with Attestor). Emit `backup_durable=true|false` and `sidecar_signed=true|false` facts during apply/restore.
  - Impacted users: Operators relying on trustworthy rollback under crash/power-loss scenarios.
  - Follow-ups: Implement durability changes in `backup.rs`; design a `backup_meta.v2` sidecar with integrity fields and dual-read.

- Invariant: Public API minimizes footguns that bypass `SafePath`
  - Assumption (from doc): Consumers use safe, high-level APIs only.
  - Reality (evidence): `src/fs/mod.rs` publicly re-exports low-level atoms: `open_dir_nofollow`, `atomic_symlink_swap`, `fsync_parent_dir` (lines 9–15), enabling external misuse that bypasses `SafePath`.
  - Gap: External callers may compromise TOCTOU safety by directly invoking low-level atoms.
  - Mitigations: Restrict re-exports to `pub(crate)` and document high-level entry points (`replace_file_with_symlink`, `restore_file`). Deprecate low-level exports first if needed.
  - Impacted users: Power users integrating at lower layers.
  - Follow-ups: Coordinate API surface change with changelog policy; add compile-fail examples in docs.

- Invariant: Secret redaction covers all volatile or sensitive fields
  - Assumption (from doc): Facts are safe to share externally.
  - Reality (evidence): `src/logging/redact.rs::redact_event()` masks helper/attestation secrets and removes timings/hashes, but does not sanitize `notes` contents from preflight rows (free-form strings) nor command-line fragments that may appear in provenance extensions.
  - Gap: Potential leakage of environment paths or args via `notes`.
  - Mitigations: Extend redaction to mask known-sensitive substrings in `notes`; add hooks for caller-provided masks. Add unit tests asserting no path-like strings leak when policy requires.
  - Impacted users: Security-conscious consumers exporting facts to external systems.
  - Follow-ups: Implement extended redaction; update SPEC §13 secret-masking policy.

- Invariant: Environment is sanitized for rescue and subprocesses
  - Assumption (from doc): Facts reflect sanitized environment.
  - Reality (evidence): `logging/audit.rs::ensure_provenance()` unconditionally inserts `provenance.env_sanitized=true` (lines 210–219) without enforcing sanitization.
  - Gap: The flag may be optimistic; consumers could misinterpret the guarantee.
  - Mitigations: Either actually sanitize PATH/locale before checks or set `env_sanitized` based on a real sanitizer result. Emit `env_vars_checked=true|false` in facts for transparency.
  - Impacted users: Environments where PATH/locale manipulation is a risk.
  - Follow-ups: Add a small sanitizer helper and thread it through preflight/apply setup.

Gap analysis in Round 2 by AI 1 on 2025-09-12 15:22 +02:00
