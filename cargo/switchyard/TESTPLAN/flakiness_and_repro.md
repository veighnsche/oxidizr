# Flakiness and Reproducibility Policy (Switchyard Library)

Defines how we keep tests deterministic, how we handle infra-related flakes, and how we prove failures are not races.

## Determinism Policy

- Fixed seeds for combination generators
  - pairwise: 4242
  - 3-wise: 314159
  - randomized input ordering: 42
- Fixed time sources where possible
  - DryRun uses `TS_ZERO` for facts (`logging/redact.rs`), guaranteed by `ts_for_mode(ApplyMode::DryRun)`.
  - Commit mode avoids comparing `ts` and `event_id`; assert only on stable fields.
- Isolated temp directories
  - Every scenario constructs a temporary root and uses `SafePath::from_rooted` for all paths.
  - Ensures no interaction with system directories; prevents permission errors across environments.
- Stable redaction
  - Compare facts after `redact_event(...)` transformation; ignore volatile fields (`duration_ms`, `lock_wait_ms`, `before_hash`, `after_hash`, `hash_alg`, provenance secrets, attestation secrets).
- Environment normalization
  - Use `SWITCHYARD_FORCE_EXDEV=1` to simulate cross-fs behavior deterministically instead of relying on host mount topology.

## Retry and Quarantine Policy

- Retries are allowed only for known infra flakes (network hiccups, transient FS permissions) and never for product logic failures.
- On failure:
  1. Collect redacted facts and FS snapshots (before/after diffs) under test temp dir.
  2. Re-run the scenario once with the same seeds and env; if it reproduces deterministically → product bug.
  3. If it does not reproduce and error references host env or Docker infra → mark flaky and quarantine.
- Quarantine protocol
  - Tag scenario with [quarantine] and remove from Bronze/Silver tiers; keep in Gold/Platinum rotation until root-caused.
  - Record an issue with the failure artifacts and redacted telemetry excerpts.

## Proving Non-Racy Behavior

- Concurrency tests pin parallelism and lock contention via explicit rival process creation time windows.
- Atomicity validated by inspecting the parent directory contents between backup creation and rename; ensure tmp names do not persist.
- Rollback validations assert FS convergence and removal of temporary files, even with crash injection.

## Seeds and Reproducible Config

- All seeds and environment toggles are recorded per scenario header in `test_selection_matrix.md` and exported alongside test artifacts.
- Time is captured as part of facts; DryRun redacts to TS_ZERO; Commit captures real time, which is ignored in assertions.

## Allowed Infra Hooks

- Disk pressure injection (Platinum): use a deterministic filler to reach a threshold; release after scenario completion.
- Lock contention: a helper holds the lock for a bounded duration; durations are fixed and <= timeout for negative cases.
- Crash/kill injection: deterministic kill at a specific step controlled by an environment variable.
