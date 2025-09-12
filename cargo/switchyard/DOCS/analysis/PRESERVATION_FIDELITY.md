# Preservation Capabilities & Restore Fidelity

**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Document what preservation dimensions are supported today versus desired (mode, uid/gid, mtime, xattrs, ACLs, file capabilities). Propose policy tiers and a concrete plan to capture/apply metadata safely.  
**Inputs reviewed:** SPEC §2.3 Safety Preconditions; SPEC §4 Preflight Diff; SPEC §5 Audit Facts; PLAN/45-preflight.md; CODE: `src/fs/{backup,restore,meta}.rs`, `src/api/preflight/mod.rs`, `src/policy/config.rs`  
**Affected modules:** `fs/backup.rs`, `fs/restore.rs`, `fs/meta.rs`, `api/preflight/mod.rs`, `policy/config.rs`

## Summary

- Current preservation support is minimal-by-design: regular file `mode` is captured and restored; symlink topology is preserved; existence (tombstone) is tracked. Owner, timestamps, xattrs, ACLs, and capabilities are not preserved.
- Preflight reports capability probes via `detect_preservation_capabilities()` and allows gating via `policy.require_preservation`.
- We propose tiered preservation modes (Basic, Extended, Full) controlled by policy, with safe, TOCTOU-free capture/apply using `*at` syscalls.

## Inventory / Findings

- Capability probe (`src/fs/meta.rs::detect_preservation_capabilities`)
  - Exposes a map: `{owner, mode, timestamps, xattrs, acls, caps}` and `preservation_supported` flag.
  - `owner`: true only when effective UID is root (parsed from `/proc/self/status`).
  - `mode`: true when metadata readable.
  - `timestamps`: true when metadata readable (heuristic; assumes `utimensat` available).
  - `xattrs`: probed via `xattr::list(path)` best-effort.
  - `acls`, `caps`: hard-coded false.

- Snapshot creation (`src/fs/backup.rs::create_snapshot`)
  - Regular files: copies bytes into timestamped backup using `openat` and sets backup `mode` via `fchmod`. Sidecar `BackupSidecar` records `prior_kind="file"` and `mode` (octal string). No owner/timestamps/xattrs/ACLs/caps recorded.
  - Symlinks: creates a symlink backup pointing to current target; sidecar records `prior_kind="symlink"` and `prior_dest`. No mode/owner/etc.
  - Missing: creates tombstone payload; sidecar `prior_kind="none"`.

- Restore (`src/fs/restore.rs`)
  - With sidecar `prior_kind=file`:
    - Renames backup payload into place via `renameat` and restores `mode` via `fchmod` when present.
  - With sidecar `prior_kind=symlink`:
    - Restores link using `atomic_symlink_swap(prior_dest)`.
  - With `prior_kind=none`:
    - Ensures absence via `unlinkat` and removes payload.
  - Owner, timestamps, xattrs, ACLs, caps are not restored.

- Preflight integration (`src/api/preflight/mod.rs`)
  - Rows include `preservation` and `preservation_supported`; policy gating can STOP when `require_preservation=true` and unsupported.

## Recommendations

1. Policy-controlled tiers
   - Add `enum PreservationTier { Basic, Extended, Full }` to `Policy` with default `Basic`.
   - Semantics:
     - Basic: capture/restore `mode` only (current behavior).
     - Extended: also capture/restore atime/mtime (`utimensat`), and xattrs (names + values) when available.
     - Full: additionally capture/restore uid/gid (`fchownat`) and best-effort ACLs/caps (platform-dependent; OK to degrade to warnings).

2. Extend sidecar schema
   - Add optional fields: `uid`, `gid`, `mtime_sec`, `mtime_nsec`, `xattrs` (map<string, base64>), `acls` (opaque JSON), `caps` (opaque JSON or string).
   - Keep `schema` at `backup_meta.v1` initially; when rolling out, version as `backup_meta.v2` and provide dual-read (v1+v2) in `read_sidecar`.

3. Safe capture/apply
   - Capture:
     - After copying bytes for files, use `fstat` to read timestamps and owner, and `xattr::list/get` for xattrs. Store in sidecar.
   - Apply:
     - After `renameat`, apply `fchownat` (if root and owner preserved), `utimensat` for times, and write back xattrs. For acl/caps, use optional adapters or platform crates if available; otherwise, log a warning fact.
   - Maintain TOCTOU invariants: operate via parent `dirfd` obtained from `open_dir_nofollow(parent)` and call `fsync(parent)` after each visibility change.

4. Gating and reporting
   - If `Policy::require_preservation` and requested `PreservationTier` > `Basic` but unsupported per probe, mark preflight STOP with `E_POLICY`.
   - Emit `preservation_applied: { owner, mode, timestamps, xattrs, acls, caps }` in apply facts for transparency (Provisional field).

5. Tests
   - Unit tests for round-trip preservation of `mode`, `mtime`, and xattrs in tmpfs.
   - Conditional tests for uid/gid restoration when running as root (skip otherwise).
   - Ensure preflight rows and apply facts include extended fields when tiers > Basic.

## Risks & Trade-offs

- Owner/ACL/caps restoration requires privileges and platform-specific crates; treat as best-effort under `Full` with clear telemetry and gating.
- Sidecar size growth when storing xattrs; mitigate with selective keys or size caps.

## Spec/Docs deltas

- SPEC §4 Preflight Diff: add optional fields for `preservation_applied` in apply facts; clarify preservation tiers and their gating.
- SPEC §5 Audit Facts: allow optional `preservation_applied` object in `apply.result` events (schema v1 additive).

## Acceptance Criteria

- Sidecar schema supports extended fields with backward-compatible read.
- Policy exposes `preservation_tier`, and apply paths conditionally restore fields.
- Tests demonstrate Extended tier restores mtime and xattrs round-trip.

## References

- SPEC: §2.3 Safety Preconditions; §4 Preflight Diff; §5 Audit Facts
- PLAN: 45-preflight.md; 60-rollback-exdev.md
- CODE: `src/fs/backup.rs`, `src/fs/restore.rs`, `src/fs/meta.rs`, `src/api/preflight/mod.rs`, `src/policy/config.rs`

## Round 1 Peer Review (AI 2, 2025-09-12 15:01 +02:00)

**Claims Verified:**
- ✅ `detect_preservation_capabilities()` function exists in `src/fs/meta.rs:75-106` and correctly probes owner (root check via `/proc/self/status`), mode, timestamps, xattrs, with ACLs/caps hard-coded false
- ✅ Backup creation in `src/fs/backup.rs:118-232` captures mode via `fchmod` and stores in `BackupSidecar.mode` as octal string for files; symlinks store `prior_dest`; missing files create tombstone with `prior_kind="none"`
- ✅ Restore logic in `src/fs/restore.rs:14-271` uses `renameat` for file restoration and `fchmod` to restore mode when present in sidecar; symlinks restored via `atomic_symlink_swap`
- ✅ Preflight integration in `src/api/preflight/mod.rs:140-144` calls `detect_preservation_capabilities()` and gates on `require_preservation` policy

**Key Citations:**
- `src/fs/meta.rs:88-91`: Owner detection via `effective_uid_is_root()` parsing `/proc/self/status`
- `src/fs/backup.rs:194-204`: Mode capture and sidecar storage for files
- `src/fs/restore.rs:128-138`: Mode restoration via `fchmod` when sidecar contains mode
- `src/api/preflight/mod.rs:142-144`: Policy gating on preservation capabilities

**Summary of Edits:** No corrections needed - all technical claims are accurately supported by the codebase. The document correctly describes current preservation support (mode only), capability probing, and proposed tiered approach.

Reviewed and updated in Round 1 by AI 2 on 2025-09-12 15:01 +02:00

## Round 2 Gap Analysis (AI 1, 2025-09-12 15:22 +02:00)

- Invariant: Extended preservation beyond mode (owner, timestamps, xattrs)
  - Assumption (from doc): Tiered preservation (Extended/Full) can be offered; current behavior is Basic (mode only).
  - Reality (evidence):
    - Capture: `src/fs/backup.rs::create_snapshot()` records only `mode` for files and `prior_dest` for symlinks; no owner/timestamps/xattrs (lines 194–205, 140–151).
    - Apply: `src/fs/restore.rs::restore_file()` restores mode via `fchmod` (lines 129–137) and does not apply owner/timestamps/xattrs.
    - Capability probe exists but is informational: `src/fs/meta.rs::detect_preservation_capabilities()` reports booleans (lines 75–106) and feeds preflight (e.g., `src/api/preflight/mod.rs` lines 140–161).
  - Gap: Consumers expecting owner/mtime/xattrs preservation will not get it even when probes say “supported”.
  - Mitigations: Add policy `preservation_tier` and extend sidecar fields (`uid/gid`, `mtime_*`, `xattrs`) with dual-read v1/v2; in apply, conditionally execute `fchownat`, `utimensat`, and xattr writes when tier and probes allow. Emit `preservation_applied{...}` in `apply.result` for transparency.
  - Impacted users: System integrators and packaging tools relying on precise metadata retention.
  - Follow-ups: SPEC §5 add optional `preservation_applied`; implement tiered restoration and unit tests for mtime/xattrs round-trip.

- Invariant: Backup/sidecar durability
  - Assumption (from doc): Backups are durable and survive crashes.
  - Reality (evidence): `write_sidecar()` uses path-based `File::create` with no `fsync(parent)`; symlink backups use `std::os::unix::fs::symlink` (path-based) (backup.rs lines 137–151, 262–270). Parent directory fsync is not performed after creating backup or sidecar.
  - Gap: Crash after creating backup/sidecar may lose artifacts (directory entry not durable), violating consumer expectations of rollback availability.
  - Mitigations: Use `open_dir_nofollow(parent)` + `symlinkat`/`openat` and call `fsync_parent_dir(backup)` after creating payload and sidecar. Add crash-sim tests (spawn child process that exits after backup) to assert artifact presence.
  - Impacted users: Operators relying on reliable rollback, especially during power loss or abrupt termination.
  - Follow-ups: Implement capability wrappers (`fs/cap.rs`) and refactor `backup.rs` accordingly; update docs to claim durability post-PR.

- Invariant: Restore remains possible after manual pruning
  - Assumption (from doc): Tombstones and sidecars guide restore even with partial artifacts.
  - Reality (evidence): `restore_file()` errors with `E_BACKUP_MISSING` when payload absent and `force_best_effort=false` (restore.rs lines 96–107). Sidecar alone does not reconstruct bytes for prior regular files.
  - Gap: If retention policies prune payloads but keep sidecars, restore of files becomes impossible.
  - Mitigations: Document that retention must keep at least one payload per target; add telemetry field `restore_ready=true|false` in preflight rows based on `has_backup_artifacts()` (`src/fs/backup.rs` lines 234–242) and teach CLI pruning to maintain readiness.
  - Impacted users: Environments with aggressive cleanup/retention jobs.
  - Follow-ups: Add `restore_ready` to preflight rows; author retention guidance in docs/CLI.

Gap analysis in Round 2 by AI 1 on 2025-09-12 15:22 +02:00
