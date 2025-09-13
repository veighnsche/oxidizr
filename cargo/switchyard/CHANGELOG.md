# Changelog

All notable changes to the Switchyard crate will be documented in this file.

This project adheres to Semantic Versioning.

## Unreleased

- Add `lock_backend` telemetry field to `apply.attempt` and `apply.result` facts (non-breaking; optional field).
- Preflight YAML exporter now preserves `preservation` and `preservation_supported` fields per SPEC.
- Preflight rows include `restore_ready` (boolean) for `RestoreFromBackup` actions.
- Deprecate legacy shim `adapters::lock_file::*`; use `adapters::lock::file::*` instead.
- Deprecate top-level `switchyard::rescue` re-export; use `switchyard::policy::rescue`.
- Add module-level documentation for `fs`, `policy`, and `api::preflight` orchestrator.
- Add compile-only public API test and JSON Schema validation test for audit facts.
- CI: guard against absolute system paths in `cargo/switchyard/tests/`.
- Docs: Update CLI Integration Guide to clarify SafePath usage and remove non-existent `prune_backups` reference.

### Removed (breaking, pre-1.0)

- Remove deprecated adapters shim `adapters::lock_file::*` (import from `adapters::lock::file::*`).
- Remove top-level alias `pub use policy::rescue` (import from `switchyard::policy::rescue`).
- Remove legacy logging helpers `audit::emit_*`; all call sites use `StageLogger` facade.
- Remove transitional restore core file `src/fs/restore/core.rs` (restore split complete).
