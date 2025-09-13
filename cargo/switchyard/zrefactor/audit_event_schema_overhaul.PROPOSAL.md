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
  - `prune.result`: required `path`, `pruned_count`, `retained_count`; optional `backup_tag`, `retention_count_limit`, `retention_age_limit_ms`

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
    "backup_tag": {"type":"string"},
    "retention_count_limit": {"type":["integer","null"]},
    "retention_age_limit_ms": {"type":["integer","null"]},
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
     "then": {"required": ["path","pruned_count","retained_count"]}}
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

## Representative example events (normative)

apply.attempt (success)

```json
{
  "schema_version": 2,
  "ts": "2025-09-13T12:00:00Z",
  "plan_id": "11111111-1111-5111-8111-111111111111",
  "stage": "apply.attempt",
  "decision": "success",
  "lock_backend": "file",
  "lock_wait_ms": 12,
  "lock_attempts": 1,
  "dry_run": false
}
```

apply.result (per-action success)

```json
{
  "schema_version": 2,
  "ts": "2025-09-13T12:00:02Z",
  "plan_id": "11111111-1111-5111-8111-111111111111",
  "action_id": "22222222-2222-5222-8222-222222222222",
  "stage": "apply.result",
  "decision": "success",
  "path": "/usr/bin/ls",
  "before_kind": "file",
  "after_kind": "symlink",
  "hash_alg": "sha256",
  "before_hash": "ab12...",
  "after_hash": "cd34...",
  "degraded": true,
  "degraded_reason": "exdev_fallback",
  "duration_ms": 4,
  "backup_durable": true
}
```

apply.result (summary failure with smoke and perf)

```json
{
  "schema_version": 2,
  "ts": "2025-09-13T12:00:03Z",
  "plan_id": "11111111-1111-5111-8111-111111111111",
  "stage": "apply.result",
  "decision": "failure",
  "path": null,
  "perf": {"hash_ms": 3, "backup_ms": 1, "swap_ms": 5},
  "error_id": "E_SMOKE",
  "exit_code": 70,
  "summary_error_ids": ["E_SMOKE","E_POLICY"]
}
```

preflight (per-action row)

```json
{
  "schema_version": 2,
  "ts": "1970-01-01T00:00:00Z",
  "plan_id": "11111111-1111-5111-8111-111111111111",
  "action_id": "22222222-2222-5222-8222-222222222222",
  "stage": "preflight",
  "decision": "success",
  "path": "/usr/bin/ls",
  "current_kind": "file",
  "planned_kind": "symlink",
  "provenance": {"uid":0,"gid":0,"pkg":"coreutils"},
  "preservation": {"owner":true,"mode":true,"timestamps":true,"xattrs":false,"acls":false,"caps":false},
  "preservation_supported": true
}
```

prune.result

```json
{
  "schema_version": 2,
  "ts": "2025-09-13T12:00:04Z",
  "plan_id": "11111111-1111-5111-8111-111111111111",
  "stage": "prune.result",
  "decision": "success",
  "path": "/usr/bin/ls",
  "backup_tag": "oxidizr",
  "pruned_count": 2,
  "retained_count": 5,
  "retention_count_limit": 10,
  "retention_age_limit_ms": null
}
```

## Migration plan

- v2 file: `SPEC/audit_event.v2.schema.json` becomes the sole supported schema.
- Update `src/logging/audit.rs` to emit `schema_version=2` universally.
- Update tests to validate representative events (`apply.attempt`, `apply.result` per-action and summary, `preflight`, `prune.result`).
- Add trybuild-type compile-time JSON examples under `tests/golden/`.

## Implementation plan (code changes)

- Add `SPEC/audit_event.v2.schema.json` (done).
- Switch `src/logging/audit.rs` to `SCHEMA_VERSION = 2`.
- Envelope injection: stop forcing `path: ""`; for v2, omit `path` by default and allow `path: null` in summaries.
- Ensure `dry_run` and `redacted` envelope flags are set from `AuditMode` (already present).
- Apply stage:
  - `apply.attempt`: always include `lock_backend`, `lock_attempts`, and optional `lock_wait_ms`.
  - `apply.result` per-action: include `hash_alg`, `before_hash`, `after_hash` when available; set `error_id`/`exit_code` on failures.
  - `apply.result` summary: include `perf` and optional `attestation`; set `error_id`/`exit_code` based on failure cause; include `summary_error_ids` chain.
- Preflight stage: per-action rows must include `current_kind` and `planned_kind` and preservation fields; summary includes `rescue_profile` and maps to `E_POLICY` on failure.
- Prune stage: include `path`, `pruned_count`, `retained_count`; optional `backup_tag`, `retention_*`.
- Provide override `SWITCHYARD_AUDIT_SCHEMA=v1|v2` for controlled rollouts.

## Acceptance checks

- `cargo test` includes a suite validating representative events against `audit_event.v2.schema.json`.
- `apply.result` summary includes `perf` and optional `attestation`; enforced by schema.
- `preflight` rows must specify `current_kind` and `planned_kind`.
- `apply.attempt` must specify `lock_backend` and `lock_attempts`.
- `prune.result` includes `path`, `pruned_count`, `retained_count` and may include `backup_tag`, `retention_count_limit`, `retention_age_limit_ms`.
- No forced empty `path` values in summaries; `path` is either omitted or `null` when not applicable.

## Backwards compatibility

- None. Pre-v1, v2 replaces v1 immediately without dual-write.
- Provide a doc snippet in SPEC outlining v1→v2 differences for integrators upgrading.

## Related

- Logging facade/instrumentation: `zrefactor/logging_audit_refactor.INSTRUCTIONS.md`
- Cohesion targets and acceptance greps: `zrefactor/responsibility_cohesion_report.md`
- SPEC alignment: `SPEC/SPEC.md` (error taxonomy, summary_error_ids)
