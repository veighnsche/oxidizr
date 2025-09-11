# Switchyard Production Readiness TODO

A living checklist of items to refine before declaring the Switchyard crate production-ready. Items are grouped to minimize risk and clarify priorities. Paths refer to `cargo/switchyard/src/`.

## Cleanups (low-risk)

- [ ] Remove placeholder doc comment in `fs/symlink.rs` (line 1: `/// placeholder`).
- [ ] Deduplicate Preflight YAML exporter: keep a single `to_yaml(...)`
  - [ ] Keep `preflight.rs::to_yaml(...)` as the canonical exporter
  - [ ] Remove `api/preflight.rs::to_yaml(...)` duplicate
  - [ ] Update docs to reference SPEC/preflight.yaml once

## Hardcoded values → constants or policy

- [ ] Unify backup tag
  - [ ] Create `constants.rs` (or a shared constants module)
  - [ ] Define `DEFAULT_BACKUP_TAG: &str = "switchyard"`
  - [ ] Use it in both `policy/config.rs` (Policy::default.backup_tag) and `fs/symlink.rs`
- [ ] Unify temp filename prefix
  - [ ] `fs/atomic.rs` uses `".{}.oxidizr.tmp"` → select a single project term (e.g., `.switchyard.tmp`)
  - [ ] Extract to `const TMP_PREFIX: &str`
- [ ] Extract fsync warn threshold
  - [ ] In `api/apply.rs`, replace magic number `fsync_ms > 50` with `const FSYNC_WARN_MS: u64 = 50`
- [ ] File lock poll sleep
  - [ ] In `adapters/lock_file.rs`, extract `thread::sleep(Duration::from_millis(25))` to `const LOCK_POLL_MS: u64 = 25`
- [ ] Rescue tool heuristic
  - [ ] In `rescue.rs`, extract the must-have list and threshold to `const` (e.g., `RESCUE_MUST_HAVE: &[&str]` and `RESCUE_MIN_COUNT: usize`)
  - [ ] Consider promoting the threshold to `Policy` if configurability is desired
- [ ] UUIDv5 namespace
  - [ ] In `types/ids.rs`, document `NS_TAG` origin and SPEC link; consider moving to shared constants
- [ ] Attestation algorithm label
  - [ ] In `api/apply.rs`, replace hardcoded `"ed25519"` with a constant or expose via `Attestor` trait (see below)

## Policy/config improvements

- [ ] Locking is required in production Commit mode
  - [ ] In `api/apply.rs`, when `ApplyMode::Commit` and no `LockManager` is configured, return `E_LOCKING` unless explicitly overridden by a policy flag
  - [ ] Add `Policy.allow_unlocked_commit: bool` (default false) to gate this behavior
  - [ ] Consider moving default lock timeout from `Switchyard::new()` into `Policy` (or a `const DEFAULT_LOCK_TIMEOUT_MS`)

## Adapters

- [ ] Smoke tests
  - [ ] Ensure integrators can pass a `SmokeTestRunner` via `with_smoke_runner(...)`
  - [ ] Document minimal expectations; current `DefaultSmokeRunner` only verifies that symlink targets match sources
- [ ] Attestor
  - [ ] Extend `Attestor` with `fn algorithm(&self) -> &'static str` (or similar) to avoid hardcoding the algorithm label in `api/apply.rs`

## Dead code / unused interfaces

- [ ] `adapters/path.rs::PathResolver` is currently unused
  - [ ] Either implement and wire it, or remove for now to avoid drift

## Test override knobs (document or feature-gate)

- [ ] Document test overrides clearly in module docs:
  - [ ] `fs/atomic.rs`: `SWITCHYARD_FORCE_EXDEV` simulates XDEV for testing
  - [ ] `rescue.rs`: `SWITCHYARD_FORCE_RESCUE_OK` toggles rescue availability
- [ ] Optional: guard override knobs behind `#[cfg(test)]` or `#[cfg(feature = "test-overrides")]`

## Preservation capability detection (future refinement)

- [ ] Improve `api/fs_meta.rs::detect_preservation_capabilities(...)`
  - [ ] Detect effective UID to set `owner` more accurately (root vs non-root)
  - [ ] Probe xattrs via `rustix`/`xattr` crate instead of defaulting to false
  - [ ] Document conservative defaults with SPEC pointer

## Logging sinks

- [ ] `logging/facts.rs::JsonlSink` is a no-op
  - [ ] Provide a simple file-backed JSONL sink under a feature flag for production integration

## Documentation polish

- [ ] Add module-level docs for `api/*` describing side-effects and fact emissions
- [ ] Add brief doc to `fs/symlink.rs` describing backup+sidecar format and atomic swap semantics

## Acceptance checks (quick wins)

- [ ] Build passes and tests still pass after constants extraction and duplicate removal
- [ ] No change in log schemas for Minimal Facts v1 unless explicitly documented
- [ ] SPEC alignment:
  - [ ] Locking behavior (E_LOCKING on Commit without lock unless overridden)
  - [ ] Determinism (timestamps zeroed in DryRun already satisfied)
  - [ ] Cross-FS degraded path support emits `degraded=true` (already implemented in facts; keep behavior)

---

Last updated: seed version based on scan at time of drafting.
