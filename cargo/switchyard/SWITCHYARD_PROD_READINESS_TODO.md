# Switchyard Production Readiness TODO

A living checklist of items to refine before declaring the Switchyard crate production-ready. Items are grouped to minimize risk and clarify priorities. Paths refer to `cargo/switchyard/src/`.

## Cleanups (low-risk)

- [x] Remove placeholder doc comment in `fs/symlink.rs` (replaced with module docs)
- [x] Deduplicate Preflight YAML exporter: keep a single `to_yaml(...)`
  - [x] Keep `preflight.rs::to_yaml(...)` as the canonical exporter
  - [x] Remove `api/preflight.rs::to_yaml(...)` duplicate
  - [ ] Update docs to reference SPEC/preflight.yaml once

## Hardcoded values → constants or policy

- [x] Unify backup tag
  - [x] Create `src/constants.rs` module
  - [x] Define `DEFAULT_BACKUP_TAG: &str = "switchyard"`
  - [x] Use it in `policy/config.rs` (Policy::default.backup_tag) and tests in `fs/symlink.rs`
- [x] Unify temp filename prefix
  - [x] Replace `".{}.oxidizr.tmp"` with a single term: `.switchyard.tmp`
  - [x] Extract to `const TMP_SUFFIX: &str`
- [x] Extract fsync warn threshold
  - [x] In `api/apply.rs`, replace magic `50` with `const FSYNC_WARN_MS: u64 = 50`
- [x] File lock poll sleep
  - [x] In `adapters/lock_file.rs`, use `const LOCK_POLL_MS: u64 = 25`
- [x] Rescue tool heuristic
  - [x] In `rescue.rs`, extract `RESCUE_MUST_HAVE: &[&str]` and `RESCUE_MIN_COUNT: usize`
  - [x] Promote the minimum threshold to `Policy` (`policy.rescue_min_count`) and plumb through preflight/apply
- [x] UUIDv5 namespace
  - [x] In `types/ids.rs`, document `NS_TAG` origin and SPEC link; moved to shared `constants`
- [x] Attestation algorithm label
  - [x] In `adapters/attest.rs`, extend `Attestor` with `fn algorithm(&self) -> &'static str`; consumed in `api/apply.rs`

## Policy/config improvements

- [x] Locking is required in Commit mode (policy-gated)
  - [x] In `api/apply.rs`, when `ApplyMode::Commit` and no `LockManager` is configured, return `E_LOCKING` unless explicitly overridden by `policy.allow_unlocked_commit`
  - [x] Add `Policy.allow_unlocked_commit: bool` (default true for dev ergonomics; set false in production presets)
  - [x] Move default lock timeout from `Switchyard::new()` into `const DEFAULT_LOCK_TIMEOUT_MS`

## Adapters

- [x] Smoke tests
  - [x] `with_smoke_runner(...)` works; `DefaultSmokeRunner` validates symlink target resolution
  - [x] Document minimal expectations for integrators (`adapters/smoke.rs` module docs)
- [x] Attestor
  - [x] Extend `Attestor` with `fn algorithm(&self) -> &'static str` to avoid hardcoding labels

## Dead code / unused interfaces

- [x] `adapters/path.rs::PathResolver` was unused — removed from public API exports (module not compiled)
  - [ ] Optionally delete the file entirely in a follow-up cleanup PR

## Test override knobs (document or feature-gate)

- [x] Document test overrides clearly in module docs:
  - [x] `fs/atomic.rs`: `SWITCHYARD_FORCE_EXDEV` simulates EXDEV for testing
  - [x] `rescue.rs`: `SWITCHYARD_FORCE_RESCUE_OK` toggles rescue availability (now documented in module docs)
- [ ] Optional: guard override knobs behind `#[cfg(test)]` or `#[cfg(feature = "test-overrides")]`

## Preservation capability detection (future refinement)

- [x] Improve `api/fs_meta.rs::detect_preservation_capabilities(...)`
  - [x] Detect effective UID to set `owner` more accurately (root vs non-root)
  - [x] Probe xattrs via the `xattr` crate instead of defaulting to false
  - [x] Document conservative defaults with SPEC pointer (module docs)

## Logging sinks

- [x] `logging/facts.rs::JsonlSink` is a no-op
  - [x] Provide a simple file-backed JSONL sink under a feature flag (`file-logging`) for production integration

## Documentation polish

- [x] Add module-level docs for `api/*` describing side-effects and fact emissions (`api/audit.rs`, `api/preflight.rs`, `api/apply.rs`)
- [x] Add brief doc to `fs/symlink.rs` describing backup+sidecar format and atomic swap semantics

## Acceptance checks (quick wins)

- [x] Build passes and tests still pass after constants extraction and duplicate removal
- [x] No change in log schemas for Minimal Facts v1 unless explicitly documented
- [x] SPEC alignment:
  - [x] Locking behavior (E_LOCKING on Commit without lock unless overridden)
  - [x] Determinism (timestamps zeroed in DryRun already satisfied)
  - [x] Cross-FS degraded path support emits `degraded=true` (already implemented in facts; keep behavior)

---

Last updated: seed version based on scan at time of drafting.
