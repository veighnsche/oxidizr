# Determinism & Stable Outputs (Planning Only)

Documents UUIDv5 namespace selection, normalization rules, and stable ordering to achieve byte-identical dry-run vs real-run outputs post-redaction.

References:

- SPEC: `cargo/switchyard/SPEC/SPEC.md §2.7 Determinism`, `§5 Audit Facts`, `§13 Schema Versioning`
- Requirements: `REQ-D1`, `REQ-D2`, `REQ-VERS1`

## Goals

- `plan_id` and `action_id` are UUIDv5 derived from normalized inputs and a stable namespace. (REQ-D1)
- Dry-run facts are byte-identical to real-run facts after timestamp redactions. (REQ-D2)
- Stable ordering of fields and collections ensures reproducible outputs. (REQ-D2)

## UUIDv5 Strategy (Planning)

- Namespace: fixed project-specific namespace UUID recorded in ADR `ADR-0006-determinism-ids.md`.
- `plan_id = uuidv5(NAMESPACE, serialize(normalized PlanInput)`
- `action_id = uuidv5(plan_id, serialize(action) + "#" + index)`

Notes:

- `serialize(action)` MUST include the action kind (e.g., EnsureSymlink vs RestoreFromBackup) and only the `SafePath.rel()` of paths to keep IDs root-agnostic.
- The `index` disambiguates multiple actions that may target the same `path.rel` with different semantics.

## Normalization Inputs

- Paths: must be `SafePath` (see `impl/25-safepath.md`) with normalized `rel` path.
- Providers/targets lists: sort lexicographically prior to hashing.
- Policy flags: serialize in a canonical order and explicit booleans for all flags.
- Remove incidental ordering by always using `BTreeMap` or equivalent for key ordering during serialization.

## Pseudocode (non-compilable)

```rust
// Planning-only pseudocode

fn compute_plan_id(input: &PlanInput) -> Uuid {
    let mut norm = normalize_input(input);    // see impl/70-pseudocode.md
    let bytes = canonical_serialize(&norm);   // stable key order, sorted collections
    uuidv5(NAMESPACE, bytes)
}

fn compute_action_id(plan_id: Uuid, action: &Action) -> Uuid {
    uuidv5(plan_id, format!("action:{}", action.path.rel))
}
```

## Stable Ordering

- Facts emission uses a serializer configured for stable key ordering. (see `impl/40-facts-logging.md`)
- Preflight rows are sorted by `path` then `action_id` prior to output. (see `impl/45-preflight.md`)

## Dry-run Redactions

- In dry-run, timestamps are set to a constant (e.g., `1970-01-01T00:00:00Z`) or normalized monotonic deltas.
- No other fields may vary between dry-run and real-run once redaction is applied.

## Tests & Evidence

- Golden fixture diff job must pass for two consecutive runs on CI with no changes.
- Dedicated test that computes `plan_id`/`action_id` for the same input twice and asserts equality.
- Additional test: compute IDs across different `root` values (with the same `rel` paths) and assert equality.
