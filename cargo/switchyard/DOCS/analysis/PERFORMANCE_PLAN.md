# Performance Profiling Plan
**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Identify hotspots, define profiling methodology, establish baselines and targets, and propose optimizations for Switchyard's core paths.  
**Inputs reviewed:** SPEC §9 Operational Bounds; PLAN/55-operational-bounds.md; CODE: `src/fs/{atomic,swap,restore,backup,meta}.rs`, `src/api/{plan,preflight,apply}/**`, `src/logging/**`  
**Affected modules:** `fs/*`, `api/*`, `logging/*`

## Round 1 Peer Review (AI 3, 2025-09-12 15:13 CEST)

**Verified Claims:**
- Hashing (`sha256_hex_of`) and directory fsyncs during `renameat` are primary IO hotspots.
- Backups/sidecars add extra IO per mutation.
- Directory scans for discovery scale with artifact count.
- The operational bounds specify fsync(parent) must occur ≤50ms after rename.
- Plan emission and apply emit facts per action.

**Citations:**
- `src/fs/meta.rs:L20-L26` - `sha256_hex_of` implementation
- `src/fs/atomic.rs:L82` - `fsync_parent_dir` call after rename
- `src/fs/backup.rs:L189-L192` - File copying in `create_snapshot`
- `src/fs/backup.rs:L277-L316` - Directory scanning in `find_latest_backup_and_sidecar`
- `src/fs/backup.rs:L25-L65` - Directory scanning in `find_previous_backup_and_sidecar`
- `SPEC/SPEC.md:L12` - Atomicity guarantee
- `SPEC/SPEC.md:L299` - fsync timing requirement
- `src/api/plan.rs` - Plan stage fact emission
- `src/api/apply.rs` - Apply stage fact emission

**Summary of Edits:**
- Added verified claims about performance hotspots based on code inspection.
- Added citations to specific implementations that confirm the described behaviors.
- Added a Round 1 Peer Review section with verification details.

Reviewed and updated in Round 1 by AI 3 on 2025-09-12 15:13 CEST

## Summary
- Hashing (`sha256_hex_of`) and directory fsyncs during `renameat` are primary IO hotspots.
- Backups/sidecars add extra IO per mutation; enumeration for discovery uses directory scans.
- JSON serialization and redaction in facts are minor CPU costs but can spike under large plans.

## Inventory / Findings
- Hotspot candidates
  - Hashing: `fs/meta.rs::sha256_hex_of()` reads entire files. Large binaries can dominate CPU and IO.
  - Fsync bursts: `fs/atomic.rs::atomic_symlink_swap()` calls `fsync_parent_dir()` after rename, and restore paths do the same; expected but measurable.
  - Backup/restore: `fs/backup.rs::create_snapshot()` copies bytes; `fs/restore.rs` performs `renameat` and optional `fchmod`, also fsync parent.
  - Directory scans: `fs/backup.rs::{find_latest_backup_and_sidecar, find_previous_backup_and_sidecar}` iterate parent; scales with artifact count.
  - Plan emission: `api/plan.rs` emits one fact per action; `apply` emits two per action; JSON building in `logging/audit.rs`.

- Measurement plan
  - Use `cargo bench` with Criterion where feasible. Add micro-benchmarks for:
    - `sha256_hex_of` over varying sizes (1KiB, 1MiB, 50MiB).
    - `atomic_symlink_swap` and `restore_file` on tmpfs/ext4 (tmpfs on CI acceptable for relative measures).
    - `create_snapshot` for files vs symlinks.
    - Fact emission throughput (serialize 10k small JSON objects).
  - Add feature flag `profiling` to compile optional benches/helper timers.

- Baselines and targets
  - Establish per-op p50/p95 on CI (tmpfs) and dev (ext4). Targets (illustrative):
    - `atomic_symlink_swap` p95 ≤ 10ms; `fsync_parent_dir` ≤ 50ms (SPEC bound).
    - Hash 50MiB file ≤ 300ms on CI.

## Recommendations
1. Hashing
   - Defer hashing for symlink targets (hash source only) as implemented; optionally add a policy to disable hashing for very large files.
   - Consider `mmap` or buffered chunk size tuning; Criterion will guide if beneficial.

2. Reduce directory scans
   - Maintain a small index file for backups (append-only) to avoid full directory read; optional and off by default.

3. Facts emission
   - Preallocate JSON maps and reuse buffers in high-volume runs; optional under `profiling` feature to compare.

4. Instrumentation
   - Add tracing spans/timers behind `profiling` feature to record `fsync_ms` and hashing durations beyond what’s already emitted.

## Risks & Trade-offs
- Reducing hashing frequency may lower forensic value; keep defaults safe and allow opt-outs only via policy.

## Acceptance Criteria
- Criterion benches land and run in CI with stable baselines.
- Performance docs include updated p95 numbers against targets.

## References
- SPEC: §9 Operational Bounds
- PLAN: 55-operational-bounds.md
- CODE: `src/fs/atomic.rs`, `src/fs/backup.rs`, `src/fs/restore.rs`, `src/fs/meta.rs`, `src/logging/audit.rs`
