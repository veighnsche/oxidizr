# ADR 0013 — Backup Tagging for Multi-CLI Isolation

Status: Accepted
Date: 2025-09-11
Owner: Product (Switchyard)

## Context

Multiple CLIs and tools may embed Switchyard on the same system. The previous placeholder design used a static backup suffix (e.g., `.oxidizr.bak`), creating a risk of cross-CLI interference and ambiguous restore behavior.

## Decision

Adopt a `backup_tag` provided via policy by the embedding application. Backups are named:

```
.<basename>.<backup_tag>.<unix_millis>.bak
```

- Restore selects the latest file matching the configured `backup_tag` and `basename` within the parent directory.
- Default tag is `"switchyard"`; embedding CLIs SHOULD override to their own value.

## Rationale

- Ensures isolation between CLIs; prevents one CLI from restoring another's backups.
- Timestamp provides uniqueness and a stable ordering for restore.
- Keeps backups colocated with targets for capability-style operations.

## Consequences

- Backup count may grow; retention policy is out of scope of this ADR and will be addressed separately.
- Directory scanning is required for restore; acceptable trade-off given simplicity and locality.

## Alternatives Considered

- Static suffix per brand: rejected due to cross-CLI conflicts.
- Global registry or database of backups: rejected due to complexity and SPOF concerns.
- Xattrs-based ownership tagging: rejected for portability and operational complexity.

## Implementation Notes

- `Policy.backup_tag: String` (default `"switchyard"`).
- Implemented in `src/fs/symlink.rs` with rustix capability-style ops and tag filtering during restore.
- Documented in `SPEC/SPEC_UPDATE_0001.md` and `DOCS/backup-restore-design.md`.
