# Audit Event Schema Overhaul — Proposal (v2)

Goal: Make the audit envelope precise, stage-aware, and forward-compatible. Introduce `schema_version = 2`, fix required/optional fields per stage, add missing fields used by code (lock_backend, lock_attempts, perf), and tighten types (UUID, hex, base64). Keep v1 reads and dual-write if needed during migration.

## Drivers (gaps in current schema v1)

- Path required globally, but summary events (e.g., `apply.result` summary) don’t always include `path`.
- No `error_id` in the schema (present in SPEC and code).
- No stage-specific shape (e.g., `apply.attempt` should include `lock_backend`, `lock_attempts`).
- Hash/signature fields lack format constraints (hex/base64).
- No `dry_run`/`redacted` envelope flags even though code has AuditMode.
- `schema_version` not required; enforce versioning going forward.

## v2 envelope (proposed)

- Required (all events):
  - `schema_version: 2`
  - `ts: string(date-time)`
  - `plan_id: string(uuid)` (UUIDv5 per SPEC)
  - `stage: enum`
  - `decision: enum { success, failure, warn }`
- Optional common envelope:
  - `action_id: string(uuid)` when event is per-action
  - `severity: enum { info, warn, error }`
  - `degraded: boolean`
  - `dry_run: boolean`
  - `redacted: boolean`
- Stage-specific fields validated via conditional subschemas (if/then):
  - `plan` action rows: `path`, allow `current_kind/planned_kind` absent
  - `preflight` rows: `path`, `current_kind`, `planned_kind`, optional `provenance`, `preservation`, `preservation_supported`
  - `apply.attempt`: `lock_backend` (string), `lock_wait_ms` (integer|null), `lock_attempts` (integer)
  - `apply.result` per-action: `path`, optional `error_id`/`exit_code`; summary: no `path`, allow `perf`, `attestation`, `error_id`, `exit_code`, `summary_error_ids`
  - `rollback` step: optional `path`
  - `rollback.summary`: `summary_error_ids` optional, `exit_code` optional
  - `prune.result`: `target_path`, `policy_used`, `pruned_count`, `retained_count`

## Tightened formats ($defs)

- `$defs.uuid`: `type: string`, `format: uuid`
- `$defs.hex`: `type: string`, `pattern: "^[a-f0-9]+$"`
- `$defs.b64`: `type: string`, `contentEncoding: base64`
- `$defs.attestation`: object with `sig_alg`, `signature ($defs.b64)`, `bundle_hash ($defs.hex)`, `public_key_id`
- `$defs.provenance`: object with `origin`, `helper`, `uid`, `gid`, `pkg`, `env_sanitized`
- `$defs.preservation`: object with `owner`, `mode`, `timestamps`, `xattrs`, `acls`, `caps`
- `$defs.perf`: object `{ hash_ms, backup_ms, swap_ms }` integers

## Example schema skeleton (excerpt)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SwitchyardAuditEventV2",
  "type": "object",
  "required": ["schema_version","ts","plan_id","stage","decision"],
  "properties": {
    "schema_version": {"type": "integer", "enum": [2]},
    "ts": {"type": "string", "format": "date-time"},
    "plan_id": {"$ref": "#/$defs/uuid"},
    "action_id": {"$ref": "#/$defs/uuid"},
    "stage": {"enum": ["plan","preflight","apply.attempt","apply.result","rollback","rollback.summary","prune.result"]},
    "decision": {"enum": ["success","failure","warn"]},
    "severity": {"enum": ["info","warn","error"]},
    "degraded": {"type": ["boolean","null"]},
    "dry_run": {"type": "boolean"},
    "redacted": {"type": "boolean"},
    "path": {"type": "string"},
    "current_kind": {"type": "string"},
    "planned_kind": {"type": "string"},
    "hash_alg": {"type":"string","enum":["sha256"]},
    "before_hash": {"$ref": "#/$defs/hex"},
    "after_hash": {"$ref": "#/$defs/hex"},
    "attestation": {"$ref": "#/$defs/attestation"},
    "provenance": {"$ref": "#/$defs/provenance"},
    "preservation": {"$ref": "#/$defs/preservation"},
    "preservation_supported": {"type":["boolean","null"]},
    "exit_code": {"type":["integer","null"]},
    "duration_ms": {"type":["integer","null"]},
    "lock_backend": {"type":"string"},
    "lock_wait_ms": {"type":["integer","null"]},
    "lock_attempts": {"type":"integer"},
    "perf": {"$ref": "#/$defs/perf"},
    "error_id": {"type":["string","null"]},
    "error_detail": {"type":["string","null"]},
    "summary_error_ids": {"type":["array","null"],"items":{"type":"string"}}
  },
  "allOf": [
    {"if": {"properties": {"stage": {"const": "plan"}}},
     "then": {"required": ["path"]}},
    {"if": {"properties": {"stage": {"const": "preflight"}}},
     "then": {"required": ["path","current_kind","planned_kind"]}},
    {"if": {"properties": {"stage": {"const": "apply.attempt"}}},
     "then": {"required": ["lock_backend","lock_attempts"],
               "properties": {"path": {"nullable": true}}}},
    {"if": {"properties": {"stage": {"const": "apply.result"}}},
     "then": {"properties": {"path": {"type": ["string","null"]}}}},
    {"if": {"properties": {"stage": {"const": "prune.result"}}},
     "then": {"required": ["target_path","pruned_count","retained_count"]}}
  ],
  "$defs": {
    "uuid": {"type": "string", "format": "uuid"},
    "hex": {"type": "string", "pattern": "^[a-f0-9]+$"},
    "b64": {"type": "string", "contentEncoding": "base64"},
    "attestation": {"type":"object","properties":{
      "sig_alg": {"type":"string","enum":["ed25519"]},
      "signature": {"$ref": "#/$defs/b64"},
      "bundle_hash": {"$ref": "#/$defs/hex"},
      "public_key_id": {"type":"string"}
    }},
    "provenance": {"type":"object","properties":{
      "origin": {"enum":["repo","aur","manual"]},
      "helper": {"type":"string"},
      "uid": {"type":"integer"},
      "gid": {"type":"integer"},
      "pkg": {"type":"string"},
      "env_sanitized": {"type":"boolean"}
    }},
    "preservation": {"type":"object","properties":{
      "owner": {"type":"boolean"},
      "mode": {"type":"boolean"},
      "timestamps": {"type":"boolean"},
      "xattrs": {"type":"boolean"},
      "acls": {"type":"boolean"},
      "caps": {"type":"boolean"}
    }},
    "perf": {"type":"object","properties":{
      "hash_ms": {"type":"integer"},
      "backup_ms": {"type":"integer"},
      "swap_ms": {"type":"integer"}
    }}
  }
}
```

Notes:
- The above shows the shape and constraints; we can refine stage-specific requirements based on final field names.
- Summary events use `path: null` and include `summary_error_ids`.

## Migration plan

- v2 file: `SPEC/audit_event.v2.schema.json` alongside v1.
- Make `src/logging/audit.rs` write v2 by default; keep a feature flag or env to dual-write v1 for a release.
- Update schema tests to validate both `apply.attempt` and `apply.result` (per-action and summary) and `preflight` rows.
- Add trybuild-type compile-time JSON examples under `tests/golden/`.

## Acceptance checks

- `cargo test` includes a suite validating representative events against `audit_event.v2.schema.json`.
- `grep -R '"path"'` usage confirms no summary events rely on globally required `path`.
- `apply.result` summary includes `perf` and optional `attestation`; enforced by schema.
- `preflight` rows must specify `current_kind` and `planned_kind`.
- `apply.attempt` must specify `lock_backend` and `lock_attempts`.

## Backwards compatibility

- Keep v1 validation for one release cycle to avoid breaking downstream tools.
- Provide a doc snippet in SPEC outlining v1→v2 differences (required fields by stage, added formats, new envelope flags).

## Related

- Logging facade/instrumentation: `zrefactor/logging_audit_refactor.INSTRUCTIONS.md`
- Cohesion targets and acceptance greps: `zrefactor/responsibility_cohesion_report.md`
- SPEC alignment: `SPEC/SPEC.md` (error taxonomy, summary_error_ids)
