# Cross-cutting — Tests & CI Pipeline

## 1) Scope

- Expand unit/integration/E2E coverage and wire CI orchestrators to validate Streams A–E.

Touched modules (validated):

- `tests/` and Rust integration tests with tmp roots
- `test-orch/` host-orchestrator and container-runner
- `src/experiments/util.rs::{create_symlinks, restore_targets}`
- `src/system/hook.rs::hook_body`

## Reuse existing infrastructure

- Use the existing Go orchestrators under `test-orch/` (host-orchestrator and container-runner). Do not introduce a parallel test runner.
- Product behavior under test must exercise the existing product modules: `src/experiments/*`, `src/experiments/util.rs`, `src/symlink/ops.rs`, `src/system/worker/*`, and `src/logging/*`. Do not add alternate symlink/logging or state paths for tests.
- Reuse the established host progress protocol (`PB> …`) and structured audit JSONL; do not add a second test log format.
- Add new YAML suites under `tests/` and new Rust tests alongside existing modules rather than creating separate test crates unless strictly required.

## 2) Rationale

- Prove safety properties and prevent regressions across distros; address flakes via cache namespacing.
- Validate safety decisions: capability/label preservation, ACL warnings, and audit attestations.

## 3) Architecture & Design

- Matrix across Arch, Manjaro, EndeavourOS; name-spaced caches per distro (already implemented in host-orchestrator).
- Add E2E YAML suites for preflight-only runs, profiles flip + rollback, AUR gating behaviors.
- Add integration tests for smoke runner and profile flip invariants.
- Add targeted tests:
  - Capability preservation: if a managed executable has `security.capability`, verify it remains after flip (requires root in container; skip gracefully when caps unsupported).
  - Label restore: when SELinux/AppArmor detected, run relabel step and assert labels on profile tree (best-effort; audit outcome checked).
  - ACL detection: ensure preflight surfaces warnings and remediation; when `--preserve-acl` set, preservation is verified.
  - Audit verification: `oxidizr-arch audit verify --op <id>` validates signature; SBOM-lite present and referenced.

## 4) Flake Mitigations & Infra Notes

- Pacman `sudo-rs` not found flake: preflight `pacman -Si` gating and retry; test suite resilience.
- Ensure locale data for derivatives when tests require locales.

## 5) Acceptance Criteria

- New suites pass consistently across supported distros with isolated caches.
- Post-flip smoke tests run and auto-rollback on induced failures; system returns healthy.
- Capabilities preserved when present; label restore attempted and audited when labeling is active.
- Preflight is default in CI; plan output includes provenance and capability/ACL findings.
- `audit verify` passes; SBOM-lite emitted for changed/untrusted targets.
- No duplicate CI/test infrastructure: tests reuse `test-orch/` programs and product logging/progress protocols.

## 6) References

- `PROJECT_PLANS/11_TESTS_&_CI_PIPELINE.md`
- `test-orch/` docs and READMEs
- `PROJECT_PLANS/SAFETY_DECISIONS_AUDIT.md`
