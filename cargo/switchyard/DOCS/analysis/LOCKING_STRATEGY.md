# Concurrency & Locking Strategy

**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Document expectations for process-level locking, timeouts, telemetry, and guidance for multi-process coordination. Provide adapter interface examples and production recommendations.  
**Inputs reviewed:** SPEC §2.5 Locking; PLAN/50-locking-concurrency.md; CODE: `src/api/apply/mod.rs`, `src/adapters/lock/{mod,file}.rs`, `src/constants.rs`  
**Affected modules:** `api/apply/mod.rs`, `adapters/lock/{mod,file}.rs`, `constants.rs`, `policy/config.rs`

## Summary

- Switchyard serializes mutating operations via a `LockManager` adapter. In production, a lock manager is required; missing lock in Commit mode yields `E_LOCKING` with bounded wait and telemetry `lock_wait_ms`.
- The default file-backed implementation `FileLockManager` uses `fs2` advisory locks with polling (`LOCK_POLL_MS`) until `DEFAULT_LOCK_TIMEOUT_MS`.
- Facts include lock acquisition metrics in `apply.attempt`. Policies control whether Commit without a lock manager is allowed.

## Inventory / Findings

- API behavior (`src/api/apply/mod.rs`)
  - On startup of `apply`: attempts to acquire process lock if `api.lock` is Some. Records wait time and emits `apply.attempt` with `lock_wait_ms`.
  - On failure to acquire within timeout: emits `apply.attempt` failure with `error_id=E_LOCKING` and a summary `apply.result` with the same mapping; returns early without mutating state.
  - If no lock manager configured and mode is Commit: enforcement depends on `Policy`:
    - If `require_lock_manager` or `!allow_unlocked_commit`, fail with `E_LOCKING`.
    - Otherwise, emit `apply.attempt` with `no_lock_manager=true` (warn path).

- Adapter traits (`src/adapters/lock/mod.rs`)
  - `trait LockManager { fn acquire_process_lock(&self, timeout_ms: u64) -> Result<Box<dyn LockGuard>>; }`
  - `trait LockGuard: Send {}` — dropping releases lock.
  - Default impl `FileLockManager` (`src/adapters/lock/file.rs`): constructor `new(PathBuf)`, timed polling with `LOCK_POLL_MS`, returns `ErrorKind::Policy` on timeout.

- Constants (`src/constants.rs`)
  - `DEFAULT_LOCK_TIMEOUT_MS = 5000`.
  - `LOCK_POLL_MS = 25`.

- Policy controls (`src/policy/config.rs`)
  - `require_lock_manager`, `allow_unlocked_commit`, `with_lock_timeout_ms` setter on `Switchyard`.

## Recommendations

1. Lock file path conventions
   - Provide a helper `Policy::default_lock_path(root: &Path) -> PathBuf` for consistent lock locations (e.g., `<root>/.switchyard/lock`), avoiding collisions across concurrent runs per root.

2. Telemetry
   - Always populate `lock_wait_ms` in the final apply summary, not only attempts, to simplify consumers.
   - Add optional `lock_backend` field to `apply.attempt` (e.g., `file`) for observability.

3. Robustness
   - Consider exponential backoff or jitter in `FileLockManager` polling to reduce contention.
   - Add a `try_with_timeout` convenience on `Switchyard` that applies `with_lock_timeout_ms` and returns self for builder chaining.

4. Multi-process guidance
   - Recommend one lock per mutable root. For package manager integrations, coordinate lock acquisition with the PM’s own lock to avoid deadlocks; document ordering: “acquire PM lock → acquire Switchyard lock → mutate → release Switchyard lock → release PM lock”.

## Risks & Trade-offs

- Advisory file locks may not be respected by all tools; for stricter environments, provide an alternative lock manager (e.g., flock-only, or PID file with fcntl + stale lock cleanup).

## Spec/Docs deltas

- SPEC §2.5: Clarify that `lock_wait_ms` appears in both attempt and summary facts; add guidance on PM lock ordering.

## Acceptance Criteria

- Rustdoc on `FileLockManager` documents timeout semantics and telemetry.
- Example snippet shows configuring a `FileLockManager` and setting a custom timeout.

## References

- SPEC: §2.5 Locking
- PLAN: 50-locking-concurrency.md
- CODE: `src/api/apply/mod.rs`, `src/adapters/lock/{mod,file}.rs`, `src/constants.rs`, `src/policy/config.rs`

## Round 1 Peer Review (AI 2, 2025-09-12 15:01 +02:00)

**Claims Verified:**
- `LockManager` trait in `src/adapters/lock/mod.rs:6-8` with `acquire_process_lock(&self, timeout_ms: u64)` method
- `FileLockManager` in `src/adapters/lock/file.rs:12-61` uses `fs2` advisory locks with polling at `LOCK_POLL_MS` intervals
- Constants in `src/constants.rs`: `DEFAULT_LOCK_TIMEOUT_MS = 5000` (L22), `LOCK_POLL_MS = 25` (L19)
- Apply module in `src/api/apply/mod.rs:57-77` tracks `lock_wait_ms` and emits `E_LOCKING` with exit code 30 on timeout
- Policy controls in `src/policy/config.rs`: `require_lock_manager`, `allow_unlocked_commit` fields (L58-66)

**Key Citations:**
- `src/adapters/lock/mod.rs:6-8`: LockManager trait definition
- `src/adapters/lock/file.rs:46-59`: Polling loop with timeout and `LOCK_POLL_MS`
- `src/api/apply/mod.rs:67-77`: Lock failure handling with `E_LOCKING` error
- `src/constants.rs:19,22`: Lock timing constants

**Summary of Edits:** All technical claims about locking implementation are accurately supported. The document correctly describes the adapter interface, file-based implementation, timing constants, and error handling.

Reviewed and updated in Round 1 by AI 2 on 2025-09-12 15:01 +02:00

## Round 2 Gap Analysis (AI 1, 2025-09-12 15:22 +02:00)

- Invariant: Lock telemetry sufficient for consumers
  - Assumption (from doc): `lock_wait_ms` in `apply.attempt` provides enough visibility.
  - Reality (evidence): Final apply summary also includes `lock_wait_ms` (`src/api/apply/mod.rs` lines 355–357). However, there is no field indicating the lock backend in use.
  - Gap: Consumers cannot distinguish file-based vs other lock backends for fleet-wide analysis.
  - Mitigations: Add `lock_backend` (e.g., "file") to `apply.attempt` and optionally to summary; document in SPEC §2.5 as additive.
  - Impacted users: Ops teams correlating lock contention to backend choice.
  - Follow-ups: Extend emission sites in `apply/mod.rs` and update JSON fixtures.

- Invariant: Lock acquisition fairness/robustness under contention
  - Assumption (from doc): File-backed lock with polling is adequate.
  - Reality (evidence): `FileLockManager` uses fixed-interval sleep (`LOCK_POLL_MS`) without backoff/jitter (`src/adapters/lock/file.rs` lines 50–56).
  - Gap: Herding and periodic contention spikes possible.
  - Mitigations: Add exponential backoff or jitter to the polling loop; surface total attempts in telemetry for diagnostics.
  - Impacted users: Highly concurrent environments.
  - Follow-ups: Prototype backoff; add a stress test that simulates N contenders.

- Invariant: Standardized lock file path prevents collisions
  - Assumption (from doc): Integrators will choose a good lock path.
  - Reality (evidence): No helper or convention exists; path is free-form (`FileLockManager::new(PathBuf)` in `src/adapters/lock/file.rs` lines 17–19).
  - Gap: Risk of path collisions across roots or processes.
  - Mitigations: Provide `Policy::default_lock_path(root)` to standardize `<root>/.switchyard/lock`; document per-root locking guidance.
  - Impacted users: Multi-tenant systems and CI pipelines.
  - Follow-ups: Add helper; update docs and examples.

- Invariant: Dev ergonomics allow Commit without LockManager by default
  - Assumption (from docs): `allow_unlocked_commit` defaults to true for development.
  - Reality (evidence): `Policy::default()` sets `allow_unlocked_commit=false` (`src/policy/config.rs` line 106). Enforcement in `apply/mod.rs` uses `require_lock_manager || !allow_unlocked_commit` (lines 101–131), so default will fail Commit without a manager.
  - Gap: Doc/code divergence may surprise developers.
  - Mitigations: Align docs/code or add compile-time examples showing explicit enablement in dev; ensure preset docs are explicit.
  - Impacted users: Developers testing Commit locally.
  - Follow-ups: Track resolution in policy doc; add a test asserting intended default.

Gap analysis in Round 2 by AI 1 on 2025-09-12 15:22 +02:00

## Round 3 Severity Assessment (AI 4, 2025-09-12 15:52 CET)

- **Title:** Insufficient Lock Telemetry for Backend Identification
  - **Category:** Observability (DX/Usability)
  - **Impact:** 2  **Likelihood:** 3  **Confidence:** 5  → **Priority:** 1  **Severity:** S4
  - **Disposition:** Implement  **LHF:** Yes
  - **Feasibility:** High  **Complexity:** 1
  - **Why update vs why not:** Lack of a `lock_backend` field in telemetry limits ops teams' ability to correlate lock contention or failures to specific backend implementations (e.g., file-based). Adding this field is a simple, low-risk enhancement for better diagnostics. The cost of inaction is minor inconvenience in fleet-wide analysis.
  - **Evidence:** `src/api/apply/mod.rs` includes `lock_wait_ms` in attempt and summary facts (lines 355–357), but no `lock_backend` field is present.
  - **Next step:** Add `lock_backend` field (e.g., "file") to `apply.attempt` and optionally to summary facts in `src/api/apply/mod.rs`. Update SPEC §2.5 to document this additive field. Implement in Round 4.

- **Title:** Lock Acquisition Lacks Fairness Under Contention
  - **Category:** Performance/Scalability
  - **Impact:** 3  **Likelihood:** 2  **Confidence:** 5  → **Priority:** 2  **Severity:** S3
  - **Disposition:** Implement  **LHF:** No
  - **Feasibility:** Medium  **Complexity:** 2
  - **Why update vs why not:** Fixed-interval polling in `FileLockManager` without backoff or jitter can lead to herding and contention spikes in highly concurrent environments, delaying lock acquisition. Adding exponential backoff or jitter improves fairness and reduces contention. The cost of inaction is potential delays in high-load scenarios.
  - **Evidence:** `src/adapters/lock/file.rs` uses fixed `LOCK_POLL_MS` sleep without backoff or jitter (lines 50–56).
  - **Next step:** Update `FileLockManager` in `src/adapters/lock/file.rs` to implement exponential backoff or jitter in the polling loop. Add telemetry for total attempts in `apply.attempt`. Plan a stress test for Round 4.

- **Title:** Risk of Lock File Path Collisions Without Standardization
  - **Category:** Missing Feature (Reliability)
  - **Impact:** 3  **Likelihood:** 2  **Confidence:** 5  → **Priority:** 2  **Severity:** S3
  - **Disposition:** Implement  **LHF:** Yes
  - **Feasibility:** High  **Complexity:** 1
  - **Why update vs why not:** Without a standardized lock file path, there's a risk of collisions across different roots or processes, potentially causing lock conflicts or deadlocks. Providing a helper for consistent paths mitigates this risk with minimal effort. The cost of inaction is potential interference in multi-tenant or CI environments.
  - **Evidence:** `FileLockManager::new(PathBuf)` in `src/adapters/lock/file.rs` accepts a free-form path with no default convention (lines 17–19).
  - **Next step:** Add `Policy::default_lock_path(root)` helper in `src/policy/config.rs` to standardize paths (e.g., `<root>/.switchyard/lock`). Update documentation and examples to promote per-root locking. Implement in Round 4.

- **Title:** Documentation and Code Divergence for `allow_unlocked_commit` Default
  - **Category:** Documentation Gap (DX/Usability)
  - **Impact:** 2  **Likelihood:** 3  **Confidence:** 5  → **Priority:** 1  **Severity:** S4
  - **Disposition:** Implement  **LHF:** Yes
  - **Feasibility:** High  **Complexity:** 1
  - **Why update vs why not:** The discrepancy between documented (`true` for dev ergonomics) and actual (`false`) default for `allow_unlocked_commit` can confuse developers testing Commit mode without a LockManager. Aligning docs or code is a simple fix to prevent minor usability issues. The cost of inaction is slight confusion during development.
  - **Evidence:** `src/policy/config.rs` docstring states `allow_unlocked_commit` defaults to `true` (lines 62–66), but `impl Default for Policy` sets it to `false` (line 106).
  - **Next step:** Update either the code to set `allow_unlocked_commit = true` by default in `src/policy/config.rs` or revise the docstring to reflect the current default. Add a test to assert the intended behavior. Implement in Round 4.

Severity assessed in Round 3 by AI 4 on 2025-09-12 15:52 CET
