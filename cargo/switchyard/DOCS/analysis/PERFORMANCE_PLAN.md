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

## Round 2 Gap Analysis (AI 2, 2025-09-12 15:23 CEST)

- **Invariant:** Predictable performance characteristics for CI/CD integration
- **Assumption (from doc):** Performance baselines and targets provide predictable timing for automation scheduling
- **Reality (evidence):** Document proposes `atomic_symlink_swap` p95 ≤ 10ms and `fsync_parent_dir` ≤ 50ms targets; however, no implementation exists for performance monitoring or enforcement; SPEC §9 operational bounds exist but no runtime validation
- **Gap:** Without performance monitoring, consumers cannot rely on timing guarantees for scheduling or SLA compliance
- **Mitigations:** Implement performance telemetry in audit facts; add policy knobs for performance thresholds with warnings/failures
- **Impacted users:** CI/CD systems with strict timing requirements and automation that schedules based on expected operation duration
- **Follow-ups:** Add performance fact fields to audit schema; implement runtime performance validation

- **Invariant:** Hash computation scalability with large files
- **Assumption (from doc):** SHA256 hashing performance scales predictably for large binaries up to 50MiB
- **Reality (evidence):** `sha256_hex_of` implementation at `src/fs/meta.rs:L20-L26` reads entire files; large binary hashing could dominate operation time; no size-based throttling or policy controls exist
- **Gap:** Large file operations may have unpredictable performance impact; consumers may encounter timeout issues with oversized files
- **Mitigations:** Add file size policy limits; implement chunked hashing with progress reporting; add hashing bypass for very large files
- **Impacted users:** Users managing large binary files and systems with strict operation timeouts
- **Follow-ups:** Implement size-aware hashing policy; add progress reporting for long-running hash operations

- **Invariant:** Directory scan performance with backup accumulation
- **Assumption (from doc):** Backup discovery performance scales reasonably with artifact count
- **Reality (evidence):** Directory scanning in `find_latest_backup_and_sidecar` at `src/fs/backup.rs:L277-L316` iterates all entries; performance degrades linearly with backup count; no optimization for high-backup-count scenarios
- **Gap:** Long-running systems with many backups may experience degraded discovery performance; no early-exit optimization for timestamp-based queries
- **Mitigations:** Implement indexed backup discovery; add retention policies to limit backup accumulation; optimize directory scanning with early exit
- **Impacted users:** Long-running services performing frequent operations and systems with accumulated backup history
- **Follow-ups:** Implement backup indexing optimization; integrate with retention strategy for performance management

Gap analysis in Round 2 by AI 2 on 2025-09-12 15:23 CEST

## Round 3 Severity Assessment (AI 1, 2025-09-12 15:44 +02:00)

- Title: No performance telemetry fields for enforcement/visibility
  - Category: Missing Feature
  - Impact: 3  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Without explicit performance fields, consumers cannot monitor regressions or enforce bounds.
  - Evidence: Facts include `fsync_ms` per action in extra via `apply/audit_fields.rs::maybe_warn_fsync`, but no summary-level performance fields or thresholds.
  - Next step: Add optional `perf` object to `apply.result` summary with aggregates (p95-ish approximations) and emit `fsync_ms` consistently; add SPEC hooks for thresholds.

- Title: Large-file hashing may cause unacceptable delays
  - Category: Performance/Scalability
  - Impact: 3  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: No
  - Feasibility: Medium  Complexity: 3
  - Why update vs why not: Hashing entire files can dominate runtime for tens of MiB binaries; need policy to cap/disable.
  - Evidence: `src/fs/meta.rs::sha256_hex_of` streams entire file; no size-gated policy today.
  - Next step: Add policy knob to disable hashing above size threshold; benchmark with Criterion to set sensible defaults.

- Title: Directory scans for backup discovery degrade with artifact count
  - Category: Performance/Scalability
  - Impact: 3  Likelihood: 2  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Backlog  LHF: No
  - Feasibility: Medium  Complexity: 3
  - Why update vs why not: Indexing or retention can alleviate; implement after retention lands.
  - Evidence: `src/fs/backup.rs::{find_latest_backup_and_sidecar (L277–316), find_previous_backup_and_sidecar (L25–65)}` iterate full directories.
  - Next step: Add optional append-only index file to accelerate discovery; or rely on retention to bound counts.

Severity assessed in Round 3 by AI 1 on 2025-09-12 15:44 +02:00
