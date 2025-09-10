# ADR Template

- Title: Feature flags and policy toggles
- Status: Accepted
- Date: 2025-09-10

## Context

Certain behaviors must be configurable via policy or compile-time flags, while preserving conservative defaults: degraded filesystem mode, strict ownership, auto-rollback, dry-run default, attestation and provenance emission.

## Decision

- Prefer runtime policy flags over compile-time cfgs for operator control:
  - `allow_degraded_fs` (default: false)
  - `strict_ownership` (default: true)
  - `disable_auto_rollback` (default: false)
  - `mode` (default: DryRun)
- Keep compile-time feature flags minimal (e.g., `attestation` optional if external crypto not desired in minimal builds).

## Consequences

+ Clear, auditable toggles surfaced in facts.
+ Conservative defaults satisfy REQ-C1/C2.
- Must document interactions to prevent unsafe combos.

## Links

- `cargo/switchyard/PLAN/10-architecture-outline.md`
- `cargo/switchyard/SPEC/SPEC.md` §§ 2.8, 2.10
