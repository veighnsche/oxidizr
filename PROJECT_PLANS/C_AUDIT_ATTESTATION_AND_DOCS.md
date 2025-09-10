# Stream C — Audit Attestation + Operator Docs

## 1) Scope

- Per-operation JSONL artifact with selective hashing and detached signature.
- CLI verifier for signatures.
- Operator docs/playbooks: recovery, files touched, exit codes, and audit verification.

Touched modules (validated):

- `src/logging/audit.rs::{audit_event_fields, audit_event, AUDIT_LOG_PATH}` (extend with op buffer/finalize)
- `src/logging/init.rs::AuditMakeWriter` (sink exists; dry-run gated via `OXIDIZR_DRY_RUN`)
- `src/system/worker/packages.rs::{query_file_owner, check_installed, repo_has_package}` (provenance)
- `src/state/mod.rs` and `src/symlink/ops.rs` for accurate target lists and timings
- New: `src/logging/attest.rs` (optional) and CLI `audit verify`

## Reuse existing infrastructure

Build attestation on top of existing logging and CLI:

- Extend `src/logging/audit.rs` with an op buffer/finalizer (and/or a small `src/logging/attest.rs`), writing JSONL and `.sig` via the existing audit sink in `src/logging/init.rs`. Do not add a second sink or writer.
- Add `oxidizr-arch audit verify` as a normal subcommand in `src/cli/{parser.rs,handler.rs}`; avoid separate binaries.
- Source provenance stays in `src/system/worker/packages.rs`; reuse `query_file_owner`, `check_installed`, and `repo_has_package` rather than re-implementing shell helpers.
- Continue to use `audit_event_fields` for structured entries; selective hashes attach as additional fields.

Acceptance (reuse):

- No new logging sink or separate process is introduced; audit bundles are emitted through the existing JSONL pipeline.
- The verifier is implemented as a CLI subcommand under `src/cli/` and uses existing modules for IO and reporting.

## Quality Requirements (Lean & Safety)

- Lean
  - Single audit sink for JSONL; no alternate file or process outputs.
  - Op buffering and finalization live in the logging module(s); verifier is a CLI subcommand, not a separate tool.
  - Minimal dependencies: Ed25519 for signatures; feature-gate signing (`--features signing`) when needed.
  - Reuse `AuditFields`; avoid creating parallel envelope structures.
- Safety
  - Tamper-evident bundles: per-op `.jsonl` plus `.sig` verifies; failures degrade gracefully (write JSONL, mark signature_status=failed).
  - Secrets masking enforced in all free-form fields; explicit allowlist of fields hashed.
  - Selective hashing with caching keyed by `(dev,inode,mtime,size)`; stable, deterministic ordering.
  - Robust verification: no network calls, clear exit codes and audit of verification results.

## Module File Structure Blueprint

- Extend existing modules
  - `src/logging/audit.rs`
    - `struct OpBuffer { id, start_ts, events: Vec<...>, artifacts: Vec<PathBuf> }`
    - `fn start_op() -> OpBuffer`
    - `fn record_event(&mut self, fields: &AuditFields)`
    - `fn finalize_op(&mut self, signing: bool) -> Result<OpSummary>` (write `audit-<op_id>.jsonl`, optional `.sig`)
  - `src/logging/attest.rs` (new, optional)
    - `fn sign(data: &[u8]) -> Result<Signature>` and `fn verify(data: &[u8], sig: &[u8]) -> Result<bool>`
    - `fn hash_path(path: &Path) -> Result<String>`; cache layer keyed by `(dev,inode,mtime,size)`
    - load keys from a predictable path or env var; feature-gated
  - `src/cli/parser.rs` / `src/cli/handler.rs`
    - Subcommand: `audit verify --op <id>`
  - `src/system/worker/packages.rs`
    - Ensure provenance fields (owner, repo presence) are attached via `AuditFields`
- Tests
  - Unit: serialization/masking, signature round-trip, hash cache eviction
  - Integration: generate bundle, verify signature, inspect selective hashes

## 2) Rationale & Safety Objectives

- Verifiable attestation per operation without heavy external infra; mask sensitive fields.

## 3) Architecture & Design

- Introduce an "Op" buffer with a unique `op_id`:
  - Collect structured events via `audit_event_fields` during the op.
  - On finalize: write `audit-<op_id>.jsonl` and an Ed25519 signature `audit-<op_id>.jsonl.sig`.
- Selective hashing:
  - Extend `AuditFields` with `before_hash`/`after_hash` as `Option<String>`.
  - Hash only targets changed this op or those from untrusted provenance (e.g., AUR/manual override).
  - Cache by `(dev,inode,mtime,size)` to avoid recomputing.
- Minimal SBOM fragment (SPDX-lite JSON):
  - `audit-<op_id>-sbom.json` with packages `{name, version, source, applets}`.
  - Include SBOM hash in a signed manifest or sign a small tarball containing JSONL + SBOM.
- CLI `oxidizr-arch audit verify --op <id>` validates signature and emits a short report.

## 4) Failure Modes & Guarantees

- If signature fails to generate: write JSONL, mark `signature_status=failed` in audit.
- Verification is idempotent; no network calls.

## 5) Preflight & Post-Verification

- Preflight: ensure audit directory writable; detect signing key presence if configured.
- Post: verify `.sig` exists and validates; emit operator-readable summary.

## 6) Docs & Operator Playbooks

- Recovery procedures (pointer re-flip, `restore_file`), PATH escape, audit verification, exit codes.
- Keep docs in sync via PR checklist.

## 7) Migration Plan

1. Phase 1: per-op JSONL without signature (wire op buffer and finalization).
2. Phase 2: add Ed25519 signatures and `audit verify`; emit SBOM-lite.

## 8) Testing Strategy

- Unit: serialization/masking, signature round-trip, hash cache behavior.
- E2E: generate op, verify signature, spot-check selective hashes.

## 9) Acceptance Criteria

- JSONL is emitted for each op; signature verifies; SBOM-lite written and referenced.
- Provenance fields present (owner, version, source) based on `pacman -Qo` and repo checks.

## 10) References

- `src/logging/{audit.rs, init.rs}`
- `src/system/worker/packages.rs::{query_file_owner, repo_has_package}`
- `PROJECT_PLANS/SAFETY_DECISIONS_AUDIT.md`
