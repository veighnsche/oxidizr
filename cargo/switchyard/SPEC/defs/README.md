# Switchyard Audit Schema — Modular Definitions

This directory contains modular JSON Schema definitions that are referenced from the main schema at:

- `../audit_event.v2.schema.json`

The goal is to keep the top-level schema ergonomic and maintainable by factoring large or stage‑specific rules into standalone, browsable files.

---

## Layout

- `defs/` (this directory)
  - `provenance.schema.json` — Provenance object (origin/helper/uid/gid/pkg, pkg_version/pkg_arch, sig_verified, env_sanitized)
  - `perf.schema.json` — Performance object (hash_ms/backup_ms/swap_ms/io bytes/timers)
  - `error.schema.json` — Error details (kind/errno/message/remediation)
  - `stages/` — Stage‑specific constraints (each file is an `if/then` rule)
    - `plan.schema.json` — plan events must include `path`
    - `preflight.schema.json` — preflight per‑action rows must include `path/current_kind/planned_kind`
    - `preflight.summary.schema.json` — preflight summary has no additional required fields
    - `apply.attempt.schema.json` — requires `lock_backend/lock_attempts`
    - `apply.result.schema.json` — no additional required fields (per‑action/summary allowed)
    - `prune.result.schema.json` — requires `path/pruned_count/retained_count`

---

## How the main schema references these files

In `../audit_event.v2.schema.json`:

- Large objects are referenced via `$defs`:
  - `"provenance": { "$ref": "./defs/provenance.schema.json" }`
  - `"perf": { "$ref": "./defs/perf.schema.json" }`
  - `"error": { "$ref": "./defs/error.schema.json" }`

- Stage constraints are imported into `$defs` and then applied in a single `allOf` block for clarity:

```json
"allOf": [
  { "$ref": "#/$defs/stage_plan" },
  { "$ref": "#/$defs/stage_preflight" },
  { "$ref": "#/$defs/stage_preflight_summary" },
  { "$ref": "#/$defs/stage_apply_attempt" },
  { "$ref": "#/$defs/stage_apply_result" },
  { "$ref": "#/$defs/stage_prune_result" }
],
"$defs": {
  "stage_plan": { "$ref": "./defs/stages/plan.schema.json" },
  "stage_preflight": { "$ref": "./defs/stages/preflight.schema.json" },
  "stage_preflight_summary": { "$ref": "./defs/stages/preflight.summary.schema.json" },
  "stage_apply_attempt": { "$ref": "./defs/stages/apply.attempt.schema.json" },
  "stage_apply_result": { "$ref": "./defs/stages/apply.result.schema.json" },
  "stage_prune_result": { "$ref": "./defs/stages/prune.result.schema.json" }
}
```

This keeps the top‑level schema readable while making stage rules discoverable as standalone documents.

---

## Editing guidelines

- Prefer additive changes; avoid breaking constraints within the same major schema version.
- Add `title` and `description` fields to new defs to improve IDE/tooling help.
- When adding a new stage:
  1) Create a `stages/<stage>.schema.json` with an `if/then` constraint.
  2) Add a `$defs.stage_<name>` that `$ref`s the file.
  3) Include `{ "$ref": "#/$defs/stage_<name>" }` in the main `allOf` list.
  4) Consider updating `$defs.stage` enum (the list of valid stage identifiers).

- When adding a large nested object:
  1) Create `defs/<object>.schema.json` with its properties.
  2) Reference it from the main schema’s `$defs` using a `$ref`.

---

## Validation

The test `tests/audit/audit_schema_v2.rs` should load `../audit_event.v2.schema.json` and validate emitted events. No separate validator is required for defs—JSON Schema `$ref` resolution will pull them in automatically when the top‑level schema is compiled.

---

## Versioning

- The filename `audit_event.v2.schema.json` denotes the major schema version (`2`).
- Minor updates (e.g., v2.1) are additive and implemented by optional fields/defs.
- Do not change `schema_version` from `2` unless a breaking v3 is introduced.
