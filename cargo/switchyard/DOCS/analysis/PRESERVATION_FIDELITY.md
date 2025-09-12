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
