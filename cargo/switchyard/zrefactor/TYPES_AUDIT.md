# Switchyard Types Utilization Audit

Date: 2025-09-12

This report analyzes whether `cargo/switchyard/src/types/` is underutilized and whether public types are "dangling" around the codebase instead of being centralized. It also proposes adjustments aligned with the layered architecture plan (types → fs → policy → adapters → api → logging) documented in `ARCH-REORG.md`.

## Scope and method

- Enumerated modules under `src/types/` and reviewed their contents.
- Searched for imports of `crate::types` across the crate to assess utilization.
- Searched for `pub struct`, `pub enum`, and `pub type` definitions outside `src/types/` to identify candidates for centralization.
- Compared current layout with the planned layered architecture and SPEC alignment (e.g., determinism UUIDv5 in ids).

## What lives in `src/types/`

- `types/errors.rs` — `ErrorKind`, `Error`, and `Result<T>` alias.
- `types/ids.rs` — deterministic `plan_id()` and `action_id()` (UUIDv5, namespace via `constants::NS_TAG`).
- `types/plan.rs` — `ApplyMode`, `LinkRequest`, `RestoreRequest`, `PlanInput`, `Action`, `Plan`.
- `types/report.rs` — `PreflightReport`, `ApplyReport`.
- `types/safepath.rs` — `SafePath` (rooted, no-dotdot, normalized path wrapper).

These form the core, layer-agnostic data model. They are broadly referenced across API, FS, policy, and adapters.

## Utilization of `crate::types`

Direct `use crate::types` imports appear in at least the following modules (sample from grep):

- `api.rs`
- `api/errors.rs`
- `api/plan.rs`
- `api/apply/mod.rs`
- `api/apply/handlers.rs`
- `api/preflight/mod.rs`
- `api/preflight/rows.rs`
- `api/rollback.rs`
- `fs/swap.rs`
- `fs/restore.rs`
- `adapters/ownership/{mod,fs}.rs`
- `adapters/lock/{mod,file}.rs`
- `adapters/{attest,smoke}.rs`
- `logging/{facts,redact}.rs`
- `policy/{gating,config}.rs`

Conclusion: `src/types/` is actively used throughout the crate and is not underutilized.

## Public types defined outside `src/types/`

Module-local public types that are currently defined outside `types/`:

- `api/errors.rs`: `ApiError`, `ErrorId`, `exit_code_for*` helpers.
  - Rationale: API-facing error surface and exit-code mapping belong in `api`. Keeping it here is appropriate.

- `policy/rescue.rs`: `RescueStatus`, `RescueError`.
  - Rationale: Policy feature-specific types; currently used only by rescue verification logic.

- `fs/mount.rs`: `MountFlags`, `MountError`, `MountInspector`, `ProcStatfsInspector`.
  - Rationale: Filesystem inspection contract and impl. Trait + impl belong in `fs`; flags/error are simple data but still FS-specific.

- `adapters/ownership/mod.rs`: `OwnershipInfo`, `OwnershipOracle`.
  - Rationale: Adapter trait and associated data. The struct is a pure data carrier with no adapter-only semantics.

- `adapters/lock/{mod,file}.rs`: `LockGuard`, `LockManager`, `FileLockManager`.
  - Rationale: Adapter contracts/impl; no shared data-only types to centralize.

- `adapters/attest.rs`: `Signature`, `Attestor`.
  - Rationale: Attestation adapter contract. `Signature` is a simple newtype over bytes.

- `adapters/smoke.rs`: `SmokeFailure`, `SmokeTestRunner`, `DefaultSmokeRunner`.
  - Rationale: Adapter contract/impl for smoke testing; error type is local to adapter.

These are mostly module-specific contracts or implementations. Centralizing traits under `types/` would blur layering (types should remain data-centric and layer-agnostic).

## Findings

- `src/types/` is already the canonical home for the core, layer-agnostic data model: plan representation, IDs, reports, `SafePath`, and base error type/alias.
- The majority of remaining public types outside `types/` are module- or layer-specific contracts (traits) or implementations, which is appropriate.
- A small number of pure data carrier structs or enums outside `types/` could optionally be re-homed to improve consistency:
  - `OwnershipInfo` (currently `adapters/ownership/mod.rs`) is a good fit for `types/ownership.rs` since it’s layer-agnostic data consumed by policy and preflight checks.
  - Optionally, `RescueStatus`/`RescueError` and `MountFlags`/`MountError` could be moved to `types/rescue.rs` and `types/mount.rs` respectively if we want cross-layer, data-only types to consistently originate from `types/`. The traits and impls should remain in their current modules.
- `preflight/rows.rs` uses stringly-typed `serde_json::Value` rows. A typed `PreflightRow` with `#[derive(Serialize)]` under `types/` would strengthen invariants and improve DX, with a thin JSON conversion at the boundary.

## Recommendations

1. Minimal centralization of data-only types (low-risk, incremental):
   - Move `OwnershipInfo` to `src/types/ownership.rs` and re-export from `types/mod.rs`.
     - Update imports in `adapters/ownership/*`, `policy/gating.rs`, and any other consumers.
   - Consider adding a typed `PreflightRow` in `src/types/preflight.rs` and use it in `api/preflight/rows.rs` (serialize at emit time).

2. Optional consistency improvements (defer if low ROI):
   - Extract `RescueStatus`/`RescueError` to `src/types/rescue.rs` while keeping verification logic in `policy/rescue.rs`.
   - Extract `MountFlags`/`MountError` to `src/types/mount.rs`; keep `MountInspector` trait and `ProcStatfsInspector` impl in `fs/mount.rs`.

3. Keep layer-specific contracts where they are:
   - `ApiError`/`ErrorId` stay in `api/errors.rs` (API boundary + exit codes mapping).
   - Adapter traits (`OwnershipOracle`, `LockManager`, `Attestor`, `SmokeTestRunner`) remain in `adapters/`.

4. Future-proofing:
   - As part of the `ARCH-REORG.md` plan, treat `src/types/` as the stable domain model crate-within-crate: no side effects, no I/O, no syscalls. This keeps compile-time layering clean and supports documentation/traceability to SPEC.

## Risk/Complexity

- Moving `OwnershipInfo`: very low risk; mechanical import updates.
- Introducing `PreflightRow`: low risk; add type + serde derive, refactor builder to construct the struct, then `serde_json::to_value` or `to_string` for emission.
- Moving `Rescue*` and `Mount*`: low to medium; update imports and ensure no circular deps.

## Appendix: references and sample matches

- `src/types/` modules: `errors.rs`, `ids.rs`, `plan.rs`, `report.rs`, `safepath.rs`.
- Sample `use crate::types` matches include: `api.rs`, `api/errors.rs`, `api/plan.rs`, `api/apply/{mod,handlers}.rs`, `api/preflight/{mod,rows}.rs`, `api/rollback.rs`, `fs/{swap,restore}.rs`, `adapters/{ownership,lock,attest,smoke}.rs`, `logging/{facts,redact}.rs`, `policy/{gating,config}.rs`.

## Conclusion

`src/types/` is not underutilized; it already anchors Switchyard’s core data model and is referenced widely. A few data-only structs/enums could be centralized to `types/` for consistency, but most remaining public types outside `types/` are correctly placed (module-specific contracts/impls). Implementing the minimal recommendations above will further align the codebase with the planned layered architecture while keeping changes low-risk.
