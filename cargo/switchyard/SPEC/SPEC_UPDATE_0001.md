# SPEC UPDATE 0001 — Safe Rust FS Layer, Rustix Adoption, and Backup Tagging

Status: Accepted
Date: 2025-09-11
Applies to: Switchyard SPEC v1.1 (Reproducible)

## Summary

This update formalizes the migration of the filesystem layer to safe Rust via `rustix`, codifies the capability-based TOCTOU-safe mutation sequence, introduces a policy-provided `backup_tag` to support multiple CLIs integrating Switchyard on the same host, and requires recording fsync timings with WARN emission when exceeding the 50ms bound.

## Motivation

- Remove all `unsafe` and raw `libc` calls to improve safety and maintainability.
- Enforce capability-style directory handles to prevent path traversal TOCTOU races.
- Support multiple CLIs that embed Switchyard without cross-interference in backups.
- Improve observability around rename/fsync latency to detect system regressions.

## Changes

1) Safe Rust syscall layer

- The crate forbids `unsafe` at the root via `#![forbid(unsafe_code)]`.
- All filesystem syscalls use `rustix`.

2) Capability-based TOCTOU-safe sequence (normative)

- Open parent with `O_DIRECTORY|O_NOFOLLOW` using `openat` relative to `CWD`.
- Perform final-component operations via `*at` calls (`symlinkat`, `renameat`, `unlinkat`).
- Fsync the parent directory immediately after rename; target bound ≤ 50ms.

3) Backup tagging for multi-CLI environments

- Backups created by mutating operations are named:
  `.basename.<backup_tag>.<unix_millis>.bak`
- The `backup_tag` is provided by the embedding application via policy.
  - Default: `"switchyard"`. CLIs SHOULD override to their own tag.
- Restore selection: only backups matching the configured `backup_tag` are considered; the latest timestamp is selected.

4) EXDEV degraded fallback

- On `renameat` returning `EXDEV`, and when policy allows degraded mode, Switchyard performs a non-atomic best-effort replacement and sets `degraded=true` in per-action facts.

5) Fsync latency telemetry and WARN

- Per-mutation fsync timing is recorded and included in `apply.result` as `duration_ms`.
- If `duration_ms > 50`, a `severity=\"warn\"` field is included in the fact.

## Backwards Compatibility

- Public API signatures remain stable at the Switchyard facade level. Internal FS methods have adjusted return values.
- Backup filenames change to include `<backup_tag>` and timestamp; restore logic now filters by tag.

## Testing, Evidence, and Gates

- Unit tests cover:
  - Atomic symlink swap and restore roundtrip using tag-filtered backups.
  - Facts emission minimal set, including degraded flag.
- Additional acceptance tests across filesystems (EXDEV) remain to be added.

## Security Considerations

- Capability handles and `O_NOFOLLOW` parent open prevent directory traversal TOCTOU races.
- Removing `unsafe` reduces memory- and UB-related risk surface.

## Open Questions & Follow-ups

- Define retention/cleanup policy for old backups.
- Add before/after SHA-256 and provenance fields to facts per SPEC §13.
- Implement formal redaction policy to replace `TS_ZERO` placeholder for dry-run determinism.
