# Placeholder → Planned Structure Migration (Planning Only)

Purpose: decouple the implementation plan from the current placeholder files in `cargo/switchyard/src/` and outline the steps to migrate to the planned modular layout without taking the placeholders as normative.

See:

- `impl/00-structure.md` — proposed module tree
- `PLAN/10-architecture-outline.md` — planned modules (placeholders to be migrated)
- `PLAN/20-spec-traceability.md` — traceability targeting the planned structure

## Current State (Placeholders)

- `src/api.rs` — oversized orchestration stub
- `src/fs_ops.rs` — mixed filesystem primitives (atomic/exdev/backup) and symlink logic
- `src/preflight.rs` — monolithic preflight stub
- `src/symlink.rs` — symlink helper stub
- `src/lib.rs` — minimal exports

These are placeholders and not normative for the final layout.

## Target Structure (Planned)

- `src/api.rs` — orchestration
- `src/preflight.rs` — gating & preflight diff
- `src/rescue.rs` — rescue profile checks
- `src/types/{plan.rs,safepath.rs,report.rs,errors.rs}`
- `src/adapters/{ownership.rs,lock.rs,path.rs,attest.rs,smoke.rs}`
- `src/logging/{facts.rs,redact.rs,provenance.rs}`
- `src/determinism/{ids.rs,ordering.rs}`
- `src/fs/{atomic.rs,backup.rs,exdev.rs,symlink.rs}`
- `src/policy/config.rs`

## Mapping

- Move `fs_ops.rs` → split across:
  - `fs/atomic.rs` (TOCTOU-safe rename flow; top-level replace entrypoints)
  - `fs/backup.rs` (backup/restore)
  - `fs/exdev.rs` (cross-FS degraded fallback)
- Move `symlink.rs` → `fs/symlink.rs` (symlink helpers used by `fs/atomic.rs`)
- Keep `preflight.rs` as a top-level module; consider submodules if it grows
- Keep `api.rs` minimal; delegate to `fs/*`, `logging/*`, `determinism/*`, and `adapters/*`

## Step-by-Step Plan (No Code Yet)

1) Create directories and empty module files under `src/` matching the target structure.
2) Move placeholder logic by responsibility:
   - Extract atomic rename + fsync to `fs/atomic.rs`
   - Extract backup/restore to `fs/backup.rs`
   - Extract EXDEV fallback into `fs/exdev.rs`
   - Keep symlink-specific helpers in `fs/symlink.rs`
3) Wire `api.rs` to call into `fs/atomic.rs` entrypoints and emit facts via `logging/facts.rs`.
4) Introduce `types/*` data structures and update imports.
5) Introduce `adapters/*` traits and swap direct calls for adapter-mediated calls.
6) Introduce `determinism/*` for UUIDv5 and stable ordering; update `api.rs` to use them.
7) Introduce `policy/config.rs` and make `ApplyMode::DryRun` the default.
8) Add `rescue.rs` checks into `preflight.rs`.
9) Run documentation and traceability checks; adjust as needed.

## CI/Gates for Migration

- Docs lint: ensure module references and links resolve
- Traceability: `SPEC/tools/traceability.py` stays green after doc updates
- No code gate yet; migration PR(s) will add compile gates only when implementation starts

## Risks & Mitigations

- Risk: Partial moves create dangling imports
  - Mitigation: perform structural moves first with empty modules; wire progressively
- Risk: Test adapters drift vs steps contract
  - Mitigation: keep `steps-contract.yaml` as source of truth; add adapter scaffolding only after structure lands

## Owners & Timeline (Planning Targets)

- Owner: Tech lead (module structure), Maintainers (splits), QA (traceability), SRE/CI (doc gates)
- Target: land the structure at start of implementation milestone (M5 handoff window per `PLAN/30-delivery-plan.md`)
