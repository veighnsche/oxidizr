# ADR Template

- Title: Preflight diff and preservation gating
- Status: Proposed
- Date: 2025-09-11

## Context

Preflight must compute a deterministic diff (YAML schema) capturing current vs planned state and policy decisions, and must gate on filesystem preservation capabilities (owner, mode, timestamps, xattrs, ACLs, caps). Failure should be fail-closed unless explicitly overridden.

## Decision

- Emit preflight entries per `/SPEC/preflight.yaml` with stable key ordering.
- Probe filesystem capabilities; if policy requires preservation that is unsupported, STOP with fail-closed unless override policy is set.
- Record provenance and notes fields for operator clarity.
- Ensure dry-run preflight is byte-identical to real-run preflight.

## Consequences

+ Early detection of unsafe environments; conservative by default.
+ Deterministic artifact suitable for golden checking.
- Requires capability detection and policy integration per platform.

## Links

- `cargo/switchyard/SPEC/preflight.yaml`
- `cargo/switchyard/SPEC/requirements.yaml` (REQ-S5)
