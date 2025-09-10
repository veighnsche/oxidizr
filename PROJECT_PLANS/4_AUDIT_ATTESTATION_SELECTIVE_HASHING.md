# AUDIT ATTESTATION & SELECTIVE HASHING

## 1) Scope (What this project changes)

- Explicit boundaries and non-goals.
  - Add per-operation JSONL with detached signature; extend structured fields; implement selective hashing policy.
  - Non-goal: per-line chained hashes.
- Source files/modules likely touched (paths + key symbols).
  - `src/logging/{init.rs,audit.rs}::{audit_event_fields, audit_event, AUDIT_LOG_PATH}`
  - `src/system/worker/packages.rs::{query_file_owner}`
  - `src/experiments/util.rs::create_symlinks`
- User-visible behavior changes.
  - New `audit-<op_id>.jsonl` and `audit-<op_id>.jsonl.sig` artifacts.

## 2) Rationale & Safety Objectives

- Why.
  - Provide verifiable audit trails with minimal operational complexity.
- Safety invariants.
  - Attestation covers all operation events; signature verifies integrity.
- Overkill → Lean replacement summary.
  - Replace per-line chain with per-operation detached signature.

## 3) Architecture & Design

- High-level approach.
  - Buffer audit events for a CLI invocation (op_id); on completion, write JSONL and sign with Ed25519. Selective hashing is computed for mutated or unowned targets; hashes stored in optional fields.
- Data model & structures.
  - Extend `AuditFields` with `before_hash`, `after_hash`, `owner_pkg`, `actor_uid/gid/user`.
- Control flow.
  - Staging (collect) → Commit (write JSONL) → Sign → Verify (separate command).
- Public interfaces.
  - CLI: `oxidizr-arch audit verify --op <op_id>`

## 4) Failure Modes & Guarantees

- Signing failure → still write JSONL; mark `signature_status=failed`.
- Hash computation failure → omit field; log `hash_status` per item.
- Idempotency: repeated verify yields consistent result.

## 5) Preflight & Post-Change Verification

- Preflight: verify writable audit dir; key presence (if in scope).
- Post: verify signature file exists and validates.

## 6) Observability & Audit

- JSONL schema fields (illustrative):
  - `ts, run_id, event, decision, target, source, duration_ms, before_hash?, after_hash?, owner_pkg?, actor_uid?, actor_user?`

## 7) Security & Policy

- Mask inputs/outputs for secrets using existing masking in `audit_event`; extend to `_fields` pipeline.
- Selective hashing policy.
  - Hash mutated or untrusted targets; `--hash` to force.

## 8) Migration Plan

- Phase 1: write per-op JSONL (no signature).
- Phase 2: sign + add verify subcommand.

## 9) Testing Strategy

- Unit: field serialization and masking.
- E2E: generate op, verify signature, spot-check hashes for changed applets.

## 10) Acceptance Criteria (Must be true to ship)

- Per-op JSONL exists; signature verifies; selective hashing respects policy.

## 11) Work Breakdown & Review Checklist

- Extend fields → buffer layer → writers → signer → verify command.

## 12) References (Repo evidence only)

- `src/logging/audit.rs::{audit_event_fields, audit_event}`
- `src/logging/init.rs::AuditMakeWriter`
- `src/system/worker/packages.rs::query_file_owner`
- `src/experiments/util.rs::create_symlinks`
- TODO_LIST_V2.md items: "Selective hashing for audit (changed or untrusted sources)", "Append-only, tamper-evident audit log", "Record actor, provenance, versions, and exit codes", "Mask sensitive fields in structured audit events"
