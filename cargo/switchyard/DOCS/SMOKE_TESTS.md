# Switchyard Smoke Tests

This document defines how we treat smoke tests in the Switchyard and provides a practical manual for integrators to use and extend the smoke-test adapter.

- Audience: integrators embedding Switchyard into their product, and contributors implementing the smoke adapter.
- Scope: policy, API surface, wiring patterns, failure semantics, CI usage, and examples.

## What are smoke tests?

Smoke tests are a minimal, fast, and deterministic set of checks that verify a plan can be safely applied and that critical invariants hold before we trust a rollout. They are not exhaustive; they are designed to catch egregious issues quickly and rollback automatically.

## Policy and principles

- Deterministic and minimal
  - Use fixed inputs, no network, bounded time budgets.
  - Output is reproducible across runs and hosts.
- Fast-path gate
  - Run on every plan application (or at least in pre-prod stages).
  - Target runtime: sub-seconds to a few seconds.
- Fail closed
  - Any smoke failure must result in an immediate, automatic rollback per policy.
  - No partial success; the system returns to a known-good state.
- Safe-by-default interfaces
  - Operate under the same TOCTOU-safe syscall regime as regular operations (open parent O_DIRECTORY|O_NOFOLLOW → openat → renameat → fsync(parent)).
- Auditability
  - Emit structured JSONL audit for start, each check, and completion (success/failure) with timings.
- Cross-filesystem awareness
  - When applicable, support degraded but safe operation with `allow_degraded_fs` and record `degraded=true` in facts.
- CI gate
  - Zero-SKIP policy for the minimal smoke suite in CI.

These principles reflect the SPEC guidance (Reproducible v1.1): minimal suite with exact commands/args, auto-rollback policy, determinism rules (stable IDs, zeroed dry-run timestamps), and bounded waits.

## Current adapter surface

The adapter lives at `cargo/switchyard/src/adapters/smoke.rs` and currently exposes a small trait.

```rust
use crate::types::plan::Plan;

#[derive(Debug)]
pub struct SmokeFailure;

pub trait SmokeTestRunner: Send + Sync {
    fn run(&self, plan: &Plan) -> std::result::Result<(), SmokeFailure>;
}

/// DefaultSmokeRunner implements a minimal, no-op smoke suite.
#[derive(Debug, Default)]
pub struct DefaultSmokeRunner;

impl SmokeTestRunner for DefaultSmokeRunner {
    fn run(&self, _plan: &Plan) -> std::result::Result<(), SmokeFailure> {
        Ok(())
    }
}
```

- `SmokeTestRunner` is the integration point. Implement this trait to provide your smoke checks.
- `DefaultSmokeRunner` is a placeholder that always passes. Replace it in production.

## Where to wire it in

- Pre-apply check: Run smoke tests after planning and preflight, before committing changes to the live system.
- Post-apply validation: Optionally run a second pass after apply to validate runtime invariants; on failure, trigger rollback.
- Rollback on failure: Any `Err(SmokeFailure)` must be treated as a policy violation that initiates rollback to the last known-good snapshot and returns a non-zero exit code.

Exact wiring depends on your application’s composition root. The typical pattern is dependency injection: pass your `SmokeTestRunner` implementation to the API/component that orchestrates `plan → preflight → apply`.

## Authoring smoke tests

- Keep checks small and cheap. Prefer presence/health checks over heavy probes.
- Avoid mutable global state or environment dependence; seed any randomness.
- Do not depend on network access unless specifically allowed and made deterministic.
- Use SafePath and TOCTOU-safe IO sequences when touching the filesystem.
- Emit audit fields for observability.

### Suggested checks (examples)

- Filesystem:
  - Target directories exist and are writable under the intended user.
  - Parents can be fsync’d following `renameat` of a temp file (simulated).
- Locking:
  - Production `LockManager` acquires within a bounded wait; record `lock_wait_ms`.
- Plan invariants:
  - No conflicting actions (e.g., same path scheduled for multiple operations in incompatible ways).
- Cross-FS:
  - If plan crosses mountpoints, validate `allow_degraded_fs` or fail fast with clear audit.

## Example: implementing a custom SmokeTestRunner

```rust
use std::time::Instant;
use crate::types::plan::Plan;
use crate::logging::audit::{audit_event_fields, AuditFields};
use crate::adapters::smoke::{SmokeTestRunner, SmokeFailure};

#[derive(Debug, Default)]
pub struct ProdSmokeRunner;

impl SmokeTestRunner for ProdSmokeRunner {
    fn run(&self, plan: &Plan) -> Result<(), SmokeFailure> {
        let start = Instant::now();
        audit_event_fields(AuditFields::builder()
            .op("smoke_start")
            .plan_id(plan.id().to_string())
            .msg("starting smoke suite")
            .build());

        // Example: ensure at least one operation is planned
        if plan.actions().is_empty() {
            audit_event_fields(AuditFields::builder()
                .op("smoke_check")
                .plan_id(plan.id().to_string())
                .msg("no actions in plan; treating as pass")
                .build());
        }

        // Example: bounded lock wait fact (pseudo-code)
        // let waited = lock_manager.peek_wait_ms();
        // audit_event_fields(AuditFields::builder()
        //     .op("smoke_fact")
        //     .fact("lock_wait_ms", waited.to_string())
        //     .build());

        audit_event_fields(AuditFields::builder()
            .op("smoke_end")
            .plan_id(plan.id().to_string())
            .elapsed_ms(start.elapsed().as_millis() as u64)
            .result("ok")
            .build());
        Ok(())
    }
}
```

Notes:
- Use `audit_event_fields` to emit structured logs. Redact secrets per policy.
- Keep the suite bounded; avoid long loops or sleeps.

## Wiring example (pseudo-code)

```rust
// In your composition root
use crate::adapters::smoke::SmokeTestRunner;

pub struct App<R: SmokeTestRunner> {
    smoke: R,
    // ... other deps
}

impl<R: SmokeTestRunner> App<R> {
    pub fn apply_with_smoke(&self, plan: &Plan) -> anyhow::Result<()> {
        self.preflight(plan)?;
        self.smoke.run(plan).map_err(|_| anyhow::anyhow!("smoke failed"))?;
        self.apply(plan)?;
        // Optional post-apply validation
        if let Err(_) = self.smoke.run(plan) {
            self.rollback(plan)?; // enforce policy
            anyhow::bail!("post-apply smoke failed; rolled back");
        }
        Ok(())
    }
}
```

## Failure semantics and rollback

- Any smoke failure is terminal for the attempt. The orchestrator must:
  - Abort the apply if pre-apply smoke fails.
  - Initiate rollback if post-apply smoke fails.
  - Record an audit event with `result=fail` and a reason.
- Exit codes map to your API error policy; treat smoke failure as a distinct error kind for alerting.

## Determinism rules

- Stable IDs: plan IDs and action IDs should be UUIDv5-derived from content, not random per-run.
- Dry-run timestamps: in dry-run, emit zeroed timestamps in facts to keep outputs diff-stable.
- No environment leakage: normalize environment-dependent paths or versions in emitted facts.

## CI integration

- Always-on gate: run smoke tests in the PR pipeline;
  - Use the minimal suite; keep total runtime low.
  - Fail the build on any smoke failure.
- Matrix: run across relevant filesystems or mounts if your product crosses FS boundaries.
- Expected fail (temporary): if you must carry a known failing case, mark it explicitly in the higher-level YAML runner as `expect: xfail` and track with an issue. Remove as soon as fixed.
- Zero-SKIP: no skipped smoke checks in CI.

## Roadmap

- SPEC §11 command set: implement a concrete command-driven smoke runner with explicit verbs (e.g., `fsync-parent`, `lock-bounded`, `path-writable`, `preflight-diff-size`).
- Auto-rollback helper: shared utility that maps smoke failure into a one-step rollback with consistent audit emission.
- Cross-FS facts: enrich emitted facts for mountpoint crossings and degraded mode.

## Appendix: references

- Adapter: `cargo/switchyard/src/adapters/smoke.rs`
- SPEC features (examples):
  - `cargo/switchyard/SPEC/features/locking_rescue.feature`
  - `cargo/switchyard/SPEC/features/atomic_swap.feature`
- Planning docs: `cargo/switchyard/PLAN/`
