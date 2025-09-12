# Feature Proposals — AI 3

Generated: 2025-09-12 16:33:01+02:00
Author: AI 3

## Feature 1: Implement `prune_backups` function (Implement now)

- **Problem statement:** No mechanism exists to clean up old backups, leading to unbounded disk usage. This is a critical missing feature for production deployments, identified during the review of `RETENTION_STRATEGY.md`.
- **User story(s):**
  - As an operator, I want to automatically prune old backups to manage disk space without manual intervention.
- **Design overview:**
  - **APIs:** A new public function `Switchyard::prune_backups(&self, target: &SafePath) -> Result<PruneResult>`.
  - **Behavior:** The function will enumerate all backups for the given `target`, sort them by timestamp, and delete older artifacts based on the `retention_count_limit` and `retention_age_limit` policies. Deletion must be atomic (pairwise unlink of sidecar and payload) and durable (by calling `fsync` on the parent directory).
  - **Telemetry/facts:** A new audit event `prune.result` will be emitted, containing `target_path`, `policy_used`, `pruned_count`, and `retained_count`.
  - **Policy flags/defaults:** Uses `retention_count_limit` and `retention_age_limit` from `Policy`.
  - **Docs changes:** Add documentation for the new `prune_backups` function and the retention policy knobs.
- **Scope (files/functions):**
  - `src/api.rs` (or `src/api/mod.rs`): Expose the new `prune_backups` function.
  - `src/fs/backup.rs`: Implement the core pruning logic.
  - `src/logging/audit.rs`: Define the new `prune.result` event.
- **Tests:**
  - **Unit:** Test the sorting and selection logic for pruning.
  - **Integration:** Create a set of backups and verify that `prune_backups` correctly removes the right number of artifacts, never deletes the last one, and that the parent directory is synced.
- **Feasibility:** High
- **Complexity:** 3
- **Effort:** M
- **Risks and mitigations:**
  - **Risk:** Accidentally deleting the wrong backups. **Mitigation:** Strong test coverage, especially for the "never delete the last backup" invariant.
- **Dependencies:**
  - SPEC Proposal `Retention Policy Knobs and Invariants`.
- **Rollout plan:**
  - Can be implemented in a single PR after the SPEC change is approved.
- **Acceptance criteria:**
  - The `prune_backups` function is available on the public API.
  - It correctly prunes backups according to count and age policies.
  - It emits a `prune.result` audit event.
- **Evidence:**
  - **Analysis:** `RETENTION_STRATEGY.md`.
  - **Code:** `src/fs/backup.rs` (contains existing backup discovery helpers).

## Feature 2: Performance Telemetry Aggregation and Emission

- **Problem statement:** Key I/O operations are known performance hotspots, but their timings are not measured or reported, hindering diagnostics. This was identified in the `PERFORMANCE_PLAN.md` review.
- **User story(s):**
  - As an SRE, I want to see detailed performance breakdowns in operation summaries to diagnose latency issues.
- **Design overview:**
  - **APIs:** No public API changes. Internal functions like `atomic_symlink_swap` and `create_snapshot` will return a small struct containing their duration.
  - **Behavior:** The `apply` and `rollback` stages will aggregate these timings from the operations they invoke. The final summary fact will include the aggregated performance data.
  - **Telemetry/facts:** The `perf` object in `apply.result` and `rollback.result` will be populated with `total_fsync_ms`, `total_hash_ms`, etc.
  - **Policy flags/defaults:** None.
  - **Docs changes:** Update SPEC §13 to document the new `perf` object fields.
- **Scope (files/functions):**
  - `src/api/apply/mod.rs`: Aggregate timings and add them to the summary fact.
  - `src/fs/atomic.rs`, `src/fs/backup.rs`, `src/fs/meta.rs`: Measure and return timings for expensive operations.
- **Tests:**
  - **Integration:** Run a full `apply` operation and assert that the `perf` object in the resulting facts contains non-zero, plausible values for the relevant timings.
- **Feasibility:** High
- **Complexity:** 2
- **Effort:** M
- **Risks and mitigations:**
  - **Risk:** Performance overhead from measuring time. **Mitigation:** Use a low-overhead timer like `std::time::Instant`. The impact should be negligible.
- **Dependencies:**
  - SPEC Proposal `Performance Telemetry in Summaries`.
- **Rollout plan:**
  - Can be implemented in a single PR.
- **Acceptance criteria:**
  - The `perf` object is correctly populated in `apply.result` facts.
  - The timings reflect the major I/O operations performed.
- **Evidence:**
  - **Analysis:** `PERFORMANCE_PLAN.md`.
  - **Code:** `src/fs/atomic.rs`, `src/fs/backup.rs` (contain the I/O hotspots).

---

Proposals authored by AI 3 on 2025-09-12 16:33:01+02:00
