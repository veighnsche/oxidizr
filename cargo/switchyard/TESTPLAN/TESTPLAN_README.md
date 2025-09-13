# Switchyard Library E2E Test Plan — Index

This README links the full set of deliverables and summarizes scope and method.

- `e2e_overview.md` — Objectives, scope, glossary, coverage goals
- `api_option_inventory.md` — Public API functions with axes/levels/boundaries and risk
- `combinatorial_model.json` — Machine-readable model for generators
- `test_selection_matrix.md` — Generation strategy and curated scenarios table
- `environment_matrix.md` — Env axes and tier mapping
- `oracles_and_invariants.md` — Deterministic oracles and invariants
- `traceability.md` — Function × Axis × Level → Scenario IDs
- `flakiness_and_repro.md` — Determinism, retries, quarantine
- `scheduling_and_cost.md` — Counts, parallelism, CI tiers
- `TODO_TESTPLAN.md` — Missing tests and TODOs

All scenarios are deterministic (seeds/time/tempdirs recorded) and auditable. The default construction path for the API is the builder (`Switchyard::builder`), and tests focus on library entry points in `src/api/`.
