# Plan, Actions, and Deterministic IDs

- Types: `PlanInput`, `Plan`, `Action`, `ApplyMode`.
- Determinism: `plan_id` is UUIDv5 over the serialized actions; each `action_id` is UUIDv5(plan_id, action+index).

Citations:
- `cargo/switchyard/src/types/plan.rs`
- `cargo/switchyard/src/types/ids.rs`
