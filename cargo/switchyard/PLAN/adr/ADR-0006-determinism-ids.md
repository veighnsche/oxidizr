# ADR Template

- Title: Determinism and UUIDv5 identifiers
- Status: Accepted
- Date: 2025-09-11

## Context

SPEC requires deterministic plan and action identifiers (UUIDv5) derived from normalized inputs and a stable namespace. Dry-run facts must be byte-identical to real-run after timestamp redaction; stable field ordering is necessary for golden fixtures.

## Decision

- Use UUIDv5 for `plan_id` and `action_id` with a stable crate-specific namespace (e.g., derived from package name and a fixed seed stored in code).
- Normalize inputs (paths, policy flags) before hashing; ensure paths are canonicalized within `SafePath` roots.
- Implement redaction policy: timestamps are zeroed or expressed as monotonic deltas in dry-run.
- Enforce stable field ordering for facts and preflight outputs.

## Consequences

+ Reproducible outputs and stable golden fixtures.
+ Easier diff-based auditing.
- Requires careful normalization and consistent serialization.

## Links

- `cargo/switchyard/SPEC/SPEC.md` §§ 2.7, 5, 12
- `cargo/switchyard/PLAN/10-architecture-outline.md`
