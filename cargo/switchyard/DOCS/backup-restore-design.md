# Backup & Restore Design (Switchyard)

Status: Draft
Date: 2025-09-11
Owner: Product (Switchyard)

## Purpose

Define unambiguous, multi-CLI-safe backup semantics for file replacement operations in Switchyard, including naming, selection, and restore behavior.

## Background

Earlier placeholder code used a static suffix `.oxidizr.bak` for backups. That implicitly coupled the library to one CLI brand (Oxidizr) and created potential cross-CLI interference when multiple CLIs integrate Switchyard on the same host. This is a product design gap rather than a code bug; backups would "work" but collide across CLIs, confusing restore behavior.

## Goals

- Isolation: Backups created by one CLI must not be considered by another.
- Determinism: Restore selects the right backup without ambiguity.
- Observability: Actions are auditable; facts include degraded mode, durations, etc.
- Safety: Avoid TOCTOU pitfalls and ensure crash-safety properties in our sequence.

## Non-Goals

- Long-term backup retention policy (to be addressed separately).
- Cross-CLI coordination beyond tag-based isolation.

## Naming Scheme (Decision)

Backups are named:

```text
.<basename>.<backup_tag>.<unix_millis>.bak
```

- `basename`: the final path component of the target being replaced.
- `backup_tag`: configured via policy by the embedding CLI (default "switchyard").
- `unix_millis`: monotonically increasing timestamp (SystemTime since UNIX_EPOCH), providing natural ordering for selection and uniqueness in concurrent operations.

Backups are placed alongside the target (same parent directory) to preserve local capability handling and restore locality.

## Creation Semantics

- For replacing a regular file with a symlink:
  - If the target exists and is a file, we create a regular-file backup by copying bytes and preserving mode.
  - If the target exists and is a symlink, we create a symlink backup pointing to the same destination.
- Backups are written using capability-style directory handles via `rustix` (`openat`, `unlinkat`, `fchmod`) to meet TOCTOU requirements.

## Selection Semantics (Restore)

- To restore, Switchyard scans the parent directory of the target for files matching `.<basename>.<backup_tag>.*.bak` and selects the latest by timestamp.
- Only backups matching the configured `backup_tag` are considered. This avoids cross-CLI interference.
- Edge cases:
  - If no matching backups exist: return NotFound unless `force_best_effort` is set.
  - If multiple backups share the same timestamp (unlikely but possible): any tie-breaker by lexical order is acceptable; practical collisions are rare.

## Restore Semantics

- For `RestoreFromBackup`, Switchyard renames the selected backup to the original basename relative to the parent directory handle and `fsync`s the parent.
- This keeps the restore atomic for the directory entry update.

## Alternatives Considered

- Single-dot static suffix (e.g., `.oxidizr.bak`): rejected due to cross-CLI interference.
- Single latest backup without timestamp: rejected due to race conditions and lack of history.
- Global backup registry file: rejected for complexity and potential single point of failure.
- Xattrs to mark ownership: rejected for portability and complexity.

## Open Questions

- Retention policy: when to prune older backups? (Policy or CLI responsibility?)
- Backup of special files (sockets, fifos) — currently out of scope; we treat as unsupported or copy metadata only.
- Extended metadata preservation (ownership, timestamps, xattrs, ACLs, caps) — governed by policy; currently limited to mode preservation on file backups.

## Mapping to Policy and SPEC

- `Policy.backup_tag: String` — default `"switchyard"`; CLIs SHOULD set their own tag.
- SPEC Update 0001 adds backup tagging to the normative spec and documents the capability-style rustix layer and unsafe ban.

## Implementation Notes

- Implemented in `src/fs/symlink.rs`: `backup_path_with_tag`, `find_latest_backup(tag)`, `replace_file_with_symlink(..., backup_tag)`, `restore_file(..., backup_tag)`.
- Uses `rustix` for `openat`, `unlinkat`, `fchmod`, and `renameat` with directory `OwnedFd` handles.
- `fsync(parent)` timing is recorded and surfaced to facts; WARN if > 50ms.

## Risks

- Directory scans for many backups may impact performance; mitigated by the simplicity of a single-directory scan and rare long histories.
- Timestamp clock anomalies could affect ordering; acceptable risk for selection; we choose the numerically largest value.
