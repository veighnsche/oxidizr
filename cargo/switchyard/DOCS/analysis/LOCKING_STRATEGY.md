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
