# ADR Template

- Title: Audit, logging, and attestation strategy
- Status: Accepted
- Date: 2025-09-10

## Context

SPEC requires structured JSONL facts for all stages, schema v1, masking policy, provenance completeness, and signed attestation bundles (ed25519). Dry-run facts must be byte-identical to real-run after timestamp redaction.

## Decision

- Emit JSON facts for `plan`, `preflight`, `apply.attempt`, `apply.result`, and `rollback` with `schema_version=1`.
- Enforce secret masking on all sinks before emission; adopt allowlist-based redaction.
- Compute SHA-256 `before_hash` and `after_hash` for mutated files.
- Generate an attestation bundle per apply and sign with ed25519; record signature fields in facts.
- Maintain stable field ordering and deterministic serialization to support golden fixtures.

## Consequences

+ High-confidence observability and auditability.
+ Enables golden fixtures and reproducibility.
- Requires schema and policy maintenance as features evolve.

## Links

- `cargo/switchyard/SPEC/audit_event.schema.json`
- `cargo/switchyard/SPEC/requirements.yaml` (REQ-O1..O7, REQ-VERS1)
