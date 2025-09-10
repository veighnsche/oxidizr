# ADR Template

- Title: EXDEV degraded mode and cross-filesystem fallback
- Status: Proposed
- Date: 2025-09-11

## Context

Cross-filesystem moves require a safe fallback because `rename` across filesystems fails with EXDEV. SPEC mandates a safe copy + fsync + rename strategy with policy-controlled degraded mode and telemetry (`degraded=true`).

## Decision

- Implement fallback path: copy staged artifact into target parent, fsync copied file and parent, then rename atomically into place.
- Gate behavior on policy:
  - If `allow_degraded_fs=true`, proceed and set `degraded=true` in facts.
  - If `allow_degraded_fs=false`, abort with `E_EXDEV` (`exdev_fallback_failed`).
- Record `degraded=true` and relevant timings in facts.

## Consequences

+ Safe operation across filesystems when policy permits.
+ Clear operator signal via telemetry and policy failure when disallowed.
- Slightly higher latency during fallback path.

## Links

- `cargo/switchyard/SPEC/SPEC.md` §§ 2.10, 10
- `cargo/switchyard/PLAN/10-architecture-outline.md`
