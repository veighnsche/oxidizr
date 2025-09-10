# Source Inventory (seed)

Modules discovered in `cargo/switchyard/src/`:

- `lib.rs` — crate exports.
  - Provisional owner: Maintainers
  - Responsibilities: expose planned API; re-exports; module wiring

- `api.rs` — high-level orchestration API.
  - Provisional owner: Tech lead
  - Responsibilities: `plan`, `preflight`, `apply`, `plan_rollback_of`; emit facts; determinism (UUIDv5); policy defaults
  - Planned functions (no code yet):
    - `fn plan(input: PlanInput) -> Plan`
    - `fn preflight(plan: &Plan) -> PreflightReport`
    - `fn apply(plan: &Plan, mode: ApplyMode, adapters: &Adapters) -> ApplyReport`
    - `fn plan_rollback_of(report: &ApplyReport) -> Plan`

- `fs_ops.rs` — safe filesystem ops and atomic swaps.
  - Provisional owner: Maintainers
  - Responsibilities: SafePath/TOCTOU sequence; atomic rename; EXDEV fallback; backup/restore

- `preflight.rs` — environment/safety preconditions.
  - Provisional owner: QA/Requirements + Maintainers
  - Responsibilities: ownership and filesystem capability gating; preservation checks; fail-closed policy

- `symlink.rs` — safe symlink replacement.
  - Provisional owner: Maintainers
  - Responsibilities: create/replace symlinks via SafePath under TOCTOU-safe sequence

Next: expand with function lists and assign provisional owners.

---

## Migration Note (planning)

Current files under `src/` are placeholders and are not normative for the final layout. Planning documents reference the planned modular structure instead. See:

- `PLAN/impl/00-structure.md` — proposed module tree
- `PLAN/impl/05-migration.md` — placeholder → planned structure mapping and steps
