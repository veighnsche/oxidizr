# Switchyard Features

This document summarizes the functionality currently implemented under `switchyard/src/`.

## Overview

Switchyard provides a small, safety-first planning and execution layer for two filesystem operations:

- Ensuring a target path is an atomic symlink to a given source.
- Restoring a previously replaced file from a local backup.

It offers planning, preflight safety checks, and atomic application with backups and dry-run support.

## Public API (api.rs)

- `FactsEmitter` trait
  - Minimal interface for emitting machine-readable events: `emit(subsystem, event, decision, fields)`.
  - Default no-op implementation; not yet used in the engine.

- `AuditSink` trait
  - Minimal line-based logging interface: `log(level, msg)`.
  - Default no-op implementation; not yet used in the engine.

- `Policy`
  - Fields:
    - `allow_roots: Vec<PathBuf>` — allowlist of root paths a target must fall under (if non-empty).
    - `forbid_paths: Vec<PathBuf>` — denylist of paths a target must not fall under.
    - `strict_ownership: bool` — present but not currently enforced in code paths.
    - `force_untrusted_source: bool` — when true, proceed with untrusted sources but surface a warning.
    - `force_restore_best_effort: bool` — when true, allow restores to succeed even when a backup is missing.

- `ApplyMode`
  - `DryRun` — simulate actions; make no changes.
  - `Commit` — apply changes (non-dry).

- Planning inputs and actions
  - `LinkRequest { source, target }`
  - `RestoreRequest { target }`
  - `PlanInput { link: Vec<LinkRequest>, restore: Vec<RestoreRequest> }`
  - `Action` variants:
    - `EnsureSymlink { source, target }`
    - `RestoreFromBackup { target }`
  - `Plan { actions: Vec<Action> }`

- Reports
  - `PreflightReport { ok, warnings, stops }`
  - `ApplyReport { executed, duration_ms, errors }`

- `Switchyard<E: FactsEmitter, A: AuditSink>`
  - `new(facts, audit, policy)` — construct a Switchyard with configured policy and (currently unused) sinks.
  - `plan(input: PlanInput) -> Plan` — converts requested link/restore operations into an ordered `Plan` of `Action`s.
  - `preflight(plan: &Plan) -> PreflightReport` — validates each `Action` with safety checks (see below). Produces warnings and stops.
  - `apply(plan: &Plan, mode: ApplyMode) -> ApplyReport` — executes actions in sequence; honors `DryRun` and `force_restore_best_effort` policy.

## Preflight Safety Checks (preflight.rs + api.rs)

For `Action::EnsureSymlink { source, target }` and `Action::RestoreFromBackup { target }`:

- Path traversal defense
  - `symlink::is_safe_path(path)` — rejects paths containing parent directory components (e.g., `..`).

- Mount suitability
  - `preflight::ensure_mount_rw_exec(path)` — requires filesystem to be mounted `rw` and not `noexec`.
  - Explicitly checks `/usr` and the `target` path.

- Immutability check
  - `preflight::check_immutable(path)` — uses `lsattr -d` to detect immutable (`i`) attribute; returns an error instructing `chattr -i`.

- Source trust (for `EnsureSymlink` only)
  - `preflight::check_source_trust(source, force)` verifies:
    - Not world-writable.
    - Root-owned (uid == 0).
    - On a suitable mount (same `rw`/`noexec` constraints).
    - Not under `$HOME` unless forced.
  - When `Policy.force_untrusted_source` is true, untrusted sources downgrade to warnings; otherwise they are stops.

- Policy path gates
  - `allow_roots`: if non-empty, `target` must start with at least one allowed root.
  - `forbid_paths`: `target` must not start with any forbidden path.

## Atomic Symlink Management (symlink.rs + fs_ops.rs)

- Path validation
  - `symlink::is_safe_path(path)` — shared by preflight and application.

- Atomic replacement with backup
  - `symlink::replace_file_with_symlink(source, target, dry_run)`:
    - No-ops when `source == target`.
    - Verifies parent directory is not a symlink via `fs_ops::open_dir_nofollow`.
    - If `target` is a symlink:
      - If it already resolves to `source` (after canonicalization and resolving relative links), operation is a no-op.
      - Otherwise, creates a symlink backup (symlink pointing to the current destination), then atomically swaps in the new symlink.
    - If `target` is a regular file:
      - Creates a regular-file backup via `fs::copy`, preserving permissions, then removes the original.
      - Ensures parent directories exist.
      - Atomically swaps in the new symlink.
    - Honors `dry_run` (does not modify the filesystem).

- Atomic swap primitive
  - `fs_ops::atomic_symlink_swap(source, target)` — creates a temporary symlink next to `target` and uses `renameat` (same-dir) to atomically replace; `fsync`s the parent directory for durability.
  - `fs_ops::open_dir_nofollow(dir)` — opens the directory with `O_NOFOLLOW` to prevent symlink traversal attacks.
  - `fs_ops::fsync_parent_dir(path)` — ensures metadata is flushed.

## Backup and Restore (symlink.rs)

- Backup naming
  - Hidden backup placed next to the target: `.<name>.oxidizr.bak` (`BACKUP_SUFFIX = ".oxidizr.bak"`).

- Restore operation
  - `symlink::restore_file(target, dry_run, force_best_effort)`:
    - If a backup exists: atomically renames backup back to `target` (same directory) and `fsync`s the parent.
    - If backup missing: returns `NotFound` unless `force_best_effort` is true.
    - Honors `dry_run`.

## Reporting and Execution Semantics (api.rs)

- `ApplyReport` records which actions executed, along with any errors and elapsed time in milliseconds.
- `PreflightReport` aggregates warnings (non-fatal) and stops (fatal). The `ok` flag is true when no stops are recorded.
- `FactsEmitter` and `AuditSink` hooks are present but not yet integrated into preflight/apply paths.

## Platform Assumptions

- Linux-oriented implementation:
  - Parses `/proc/self/mounts`.
  - Uses `libc::renameat`, `O_NOFOLLOW`, and `lsattr`.
  - Relies on UNIX-specific metadata (e.g., `MetadataExt`).

## Notes and Current Limitations

- `Policy.strict_ownership` is defined but currently unused in enforcement logic.
- Facts and audit hooks are not yet wired into execution; all reporting is via returned structs, not logs.
- Operations are focused on symlink replacement and backup-based restoration; broader file ops are out of scope.
