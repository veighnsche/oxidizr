# REFACTOR_EXECUTION_PLAN.md

A staged, low-risk sequence of PRs to refactor within `src/` while preserving behavior, CLI, logs, and exit codes. No external crate extraction yet; this keeps the tree coherent and prepares for a future `oxidizr-core`.

## Step 0 — Guardrails and contracts (docs-only, config)

- Scope: Add CONTRIBUTING section “Authoritative Modules & Reuse Rules” mirroring `PROJECT_PLANS/README.md`. Add CI grep rules to forbid `which::which` outside `fs_ops.rs` and to forbid new ad-hoc symlink ops/log sinks.
- Risks: None (non-functional).
- Acceptance: CI passes; rules trigger on deliberate violations in a test commit.
- Rollback: Remove docs and CI rule changes.

## Step 1 — Centralize PATH search behind Worker.which (done)

- Scope: Ensure all call sites go through `Worker.which()` (already refactored in `system/worker/{aur.rs,packages.rs}`); keep feature-gated option to swap to internal `path_search` later.
- Risks: Minimal.
- Acceptance: `cargo build` and existing tests; Verify no `which::which` usages outside `fs_ops.rs`.
- Rollback: Revert imports.

## Step 2 — Add pointer flip primitive (no behavior change yet)

- Scope: Add `rename_active_pointer(active, new_target)` to `src/symlink/ops.rs` leveraging `O_DIRECTORY|O_NOFOLLOW` parent + `renameat` + parent fsync. Add unit tests verifying atomicity (Linux-only guarded) and path safety.
- Risks: Introducing a new API; not referenced by product yet.
- Acceptance: Unit tests green; no product behavior change.
- Rollback: Remove the new function.

## Step 3 — Introduce Core namespace boundaries (logical only)

- Scope: Create `src/core/` module namespace re-exporting existing authorities without moving files:
  - `core::symlink` -> `crate::symlink`
  - `core::fs_checks` -> `crate::system::fs_checks`
  - `core::path` -> adapter to `Worker.which()` (temporary) or direct feature-gated `path_search`
  - `core::audit` -> `crate::logging::{audit.rs, init.rs}`
  - `core::state` -> `crate::state`
  - `core::lock` -> `crate::system::lock`
- Risks: Module visibility and import churn.
- Acceptance: All imports compile; behavior unchanged.
- Rollback: Remove re-exports; search/replace back to previous imports.

## Step 4 — Worker-to-Core dependency alignment

- Scope: Update `Worker` methods to call Core re-exports (from Step 3) explicitly. Keep `Worker` in product layer; prohibit Core from importing Worker.
- Risks: None if re-exports correct.
- Acceptance: Build & tests pass; no circular deps.
- Rollback: Revert imports.

## Step 5 — Audit compatibility and schema snapshot tests

- Scope: Add snapshot tests for typical flows (enable coreutils, flip checksums, disable sudo-rs) asserting JSONL lines contain the expected envelope and fields.
- Risks: Test stability; time fields differ (normalize timestamps).
- Acceptance: CI green; snapshots updated on intentional schema changes only.
- Rollback: Remove snapshots.

## Step 6 — Preflight builder (Stream B scaffolding, read-only)

- Scope: Implement `PreflightItem`, builder, and renderer in `experiments/util.rs`. Wire CLI `--preflight` flag to run builder + renderer without mutating state; default posture can come later.
- Risks: Minimal; read-only.
- Acceptance: Unit tests for builder/renderer; E2E: preflight-only run prints plan and exits.
- Rollback: Feature-flag or revert.

## Step 7 — Provenance tightening (Stream D alignment)

- Scope: Ensure `install_package` emits provenance via `audit_event_fields` consistently (helper name, cmd, rc) and that `repo_has_package`/`extra_repo_available` decisions are audited.
- Risks: Log volume; mitigated with INFO-level policy.
- Acceptance: Logs present in E2E; acceptance messages match README guarantees.
- Rollback: Adjust log levels or fields only.

## Step 8 — Optional: internal path_search implementation (Stream E)

- Scope: Implement `path_search` in `fs_ops.rs` and feature-gate the external `which` crate. Add a unit corpus to compare behaviors.
- Risks: Edge-case divergence.
- Acceptance: Unit corpus matches external crate; E2E unaffected.
- Rollback: Re-enable external crate feature.

## Step 9 — Attestation scaffolding (Stream C, gated)

- Scope: Add `logging/attest.rs` with signing/verify and a small `OpBuffer` to `audit.rs` behind a feature flag. No product CLI yet.
- Risks: Crypto key handling; mitigate by feature gate and docs.
- Acceptance: Unit tests for sign/verify, deterministic hashing; no runtime effect when feature disabled.
- Rollback: Remove gated files.

## Step 10 — Profiles pointer flip integration (Stream A, incremental)

- Scope: Use `rename_active_pointer` when profile layout exists; keep current `/usr/bin` symlink mode as default. Add a hidden flag or env to opt into profile mode in tests only.
- Risks: Behavior drift if enabled accidentally.
- Acceptance: Existing default path unchanged; gated mode E2E passes additional profile tests.
- Rollback: Disable flag; revert wiring.

## Step 11 — Finalize Core boundary (internal only)

- Scope: Move core authorities into `src/core/` physically (symlink ops, fs checks, audit, state, lock, path). Update imports. Keep crate as one package for now.
- Risks: Import paths in many files; mitigated by IDE/search.
- Acceptance: Build & tests pass; call graph intact.
- Rollback: Move files back or re-export.

## Step 12 — Extraction feasibility (no code move)

- Scope: Draft `core/Cargo.toml` (not added to workspace yet) to validate dependency independence (no Worker/experiments deps). Ensure error types and logging macros are Core-local or exported cleanly.
- Risks: None (scoping exercise).
- Acceptance: `cargo check -p oxidizr-arch` unaffected; `core/` draft builds independently in isolation tests.
- Rollback: Remove draft.

---

## Acceptance tests mapping

- Existing suites (tests/ YAML + test-orch) continue to run across steps; critical checkpoints after Steps 2, 3, 6, 7, 8, 10, 11.
- Additional unit tests accompany new APIs.

## Rollback general path

- Each step is self-contained and revertible; no large renames without prior re-exports.
- Keep changes behind feature flags where risk exists (attestation, path_search, profile flip mode).
