# Plan, Actions, and Deterministic IDs

- Types: `PlanInput`, `Plan`, `Action`, `ApplyMode`.
- Determinism: `plan_id` is UUIDv5 over the normalized inputs using a stable project namespace; each `action_id` is UUIDv5(plan_id, action+index).
- Namespace: a project-defined constant (see `src/constants.rs::NS_TAG`) ensures stability across runs/environments.
- Dry-run parity: redaction zeros timestamps so DryRun and Commit facts remain byte-identical after redaction.

Traceability
- See SPEC §2.7 for determinism; §5 for facts envelope fields.
- `SPEC/traceability.md` maps requirements to scenarios for coverage.

Citations:
- `src/types/plan.rs`
- `src/types/ids.rs`
