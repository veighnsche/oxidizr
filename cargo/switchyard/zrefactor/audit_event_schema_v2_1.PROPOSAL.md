# Audit Event Schema v2.1 — Proposal (minor upgrade)

Goal: tighten correctness, improve observability, and future‑proof the event stream without breaking v2 clients. v2.1 remains within the v2 major—strictly additive or relaxing constraints; no removals, no changes to existing field semantics.

This document enumerates gaps in v2 and proposes v2.1 additions, examples, and acceptance. It also maps each change to concrete implementation points in the codebase and schema.

---

## Drivers (identified gaps in v2)

- Missing unique event identifier (dedup/correlation across sinks).
- No run/session correlation beyond `plan_id` (tie facts to a single CLI/API invocation).
- Weak resource model: only a `path` string; lacks inode/mount/device for stable resource identity.
- Coarse perf object: no IO bytes, no sub‑timers beyond hash/backup/swap.
- Error taxonomy surface lacks raw OS error info and remediation hints.
- Provenance lacks package version/arch/signature verification result.
- Attestation object lacks key identity provenance and certificate chain.
- Redaction is boolean only; lacks structured details of what was redacted and why.
- Stage model ambiguous for summaries (preflight, plan); no event sequencing.
- Hash model constrained to a single alg; cannot report multiple algs or sizes.
- Host/process/actor metadata absent; hard to triage environment regressions.

---

## Envelope (additive)

Add the following optional fields to the common envelope:

- `event_id: string(uuid)` — unique per event (UUIDv7 recommended).
- `run_id: string` — correlates all events from one API/CLI invocation (distinct from deterministic `plan_id`).
- `switchyard_version: string` — semver of the library.
- `build: { git_sha: string, rustc: string }` — build provenance (optional).
- `host: { hostname: string, os: string, kernel: string, arch: string }` — environment snapshot (optional).
- `process: { pid: integer, ppid: integer }` — minimal process metadata (optional).
- `actor: { euid: integer, egid: integer }` — effective ids (optional).
- `redaction: { applied: boolean, rules: [string] }` — details of redaction beyond boolean (optional).
- `seq: integer` — monotonic sequence per `plan_id` (or per `run_id`) to aid ordering (optional).

Schema impact: new optional properties; no effect on v2 validators.

Code touchpoints:

- `src/logging/audit.rs::redact_and_emit(...)` — inject `event_id` (if not provided), `switchyard_version`, and optionally host/process/actor via helpers.
- `src/logging/mod.rs` — expose helpers/utilities if needed.

---

## Resource model (additive, preferred)

Introduce a `resource` object to complement (not replace) `path`:

```json
"resource": {
  "path": "/usr/bin/ls",
  "rel": "usr/bin/ls",
  "inode": 123456,
  "device": "8:1",
  "mount": "/usr"
}
```

Notes:

- `path` stays for quick filtering. `resource` adds stable identity across renames and improves post‑mortem analysis.
- For symlink operations, inode data may be unavailable pre‑mutation; emit best‑effort.

Schema impact: optional `resource` object in `$defs` (future consumers can adopt without breaking v2 clients).

Code touchpoints:

- `src/fs/meta.rs` (or suitable helper) to probe inode/device/mount.
- `api/*` stages attach `resource` where available (especially apply/preflight rows).

---

## Perf model (expanded)

Expand perf to capture IO and sub‑timers:

```json
"perf": {
  "hash_ms": 3,
  "backup_ms": 1,
  "swap_ms": 5,
  "io_bytes_read": 4096,
  "io_bytes_written": 8192,
  "timers": {
    "fsync_ms": 1,
    "snapshot_ms": 0,
    "integrity_ms": 2
  }
}
```

Schema impact: all fields optional; keep existing ones as‑is. Add integer `io_bytes_*` and open‑ended `timers` map of integer milliseconds.

Code touchpoints:

- `api/apply/handlers.rs` — record `fsync_ms`, propagate into `timers.fsync_ms` and maintain `duration_ms`.
- `fs/backup/` and `fs/restore/` — attach `snapshot_ms`, `integrity_ms` if measured.

---

## Error taxonomy (richer surface)

Add optional details while keeping `error_id` + `exit_code` unchanged:

```json
"error": {
  "kind": "io|policy|locking|integrity|smoke|unknown",
  "errno": 18,
  "message": "EXDEV: cross-device link",
  "remediation": "enable degraded fallback or align mounts"
}
```

Schema impact: new `error` object; `error_id`/`exit_code` remain top‑level for quick routing.

Code touchpoints:

- `api/errors.rs` — map `ErrorId` to `error.kind` and include `errno` where available.
- `api/apply/*` — populate `error.message` and context‑specific remediation hints.

---

## Provenance/attestation (expanded)

- `provenance.pkg_version: string` and `provenance.pkg_arch: string`.
- `provenance.sig_verified: boolean|null` (signature verification when applicable).
- `attestation` add:
  - `key_id: string`, `cert_chain: [string]` (PEM or reference), `signature_format: string`.

Schema impact: optional properties within existing objects.

Code touchpoints:

- `adapters/attest.rs` — extend `build_attestation_fields` to include `key_id`/chain when available.
- `adapters/ownership/*` — surface package version/arch if resolvable.

---

## Hash model (multi‑alg)

Add `hashes` as an optional array, keeping `hash_alg`/`before_hash`/`after_hash` for compatibility:

```json
"hashes": [
  { "role": "before", "alg": "sha256", "value": "...", "size": 1234 },
  { "role": "after",  "alg": "sha256", "value": "...", "size": 1234 }
]
```

Schema impact: optional `hashes` array with `$defs.hash`.

Code touchpoints:

- `fs/meta::sha256_hex_of` — keep; optionally add multi‑alg helper.
- `api/apply/handlers.rs` — populate `hashes` when enabled.

---

## Stage model refinements

- Add `preflight.summary` (preferred) for the summary row without `path/current_kind/planned_kind`.
- Consider `plan.summary` for parity (explicit counts and deterministic ordering status).
- Optional event delineation: `apply.begin` / `apply.end` as wrappers around per‑action events for large plans (stream‑friendly).

Schema impact: add new `stage` enum variants; keep existing constraints.

Code touchpoints:

- `logging/audit.rs::Stage` and `StageLogger` — add new builders.
- `api/preflight/mod.rs` & `api/plan.rs` — emit new summary events.

---

## Host/process/actor metadata (optional helpers)

Add best‑effort environment snapshot via helpers under `logging/`.

Schema impact: `host`, `process`, `actor` optional objects.

Code touchpoints:

- `logging/audit.rs::redact_and_emit` — attach when feature `envmeta` is enabled.

---

## Representative examples

apply.result (per‑action failure with rich error)

```json
{
  "schema_version": 2,
  "event_id": "018f3b8c-5f6f-7b2a-b3c8-8f2b51f5b9a1",
  "run_id": "session-20250913-1200-abc",
  "switchyard_version": "0.1.0",
  "plan_id": "11111111-1111-5111-8111-111111111111",
  "action_id": "22222222-2222-5222-8222-222222222222",
  "stage": "apply.result",
  "decision": "failure",
  "path": "/usr/bin/ls",
  "resource": {"path":"/usr/bin/ls","rel":"usr/bin/ls","inode":123456,"device":"8:1","mount":"/usr"},
  "before_kind": "file",
  "after_kind": "symlink",
  "hashes": [
    {"role":"before","alg":"sha256","value":"ab..","size":1024},
    {"role":"after","alg":"sha256","value":"cd..","size":1024}
  ],
  "perf": {"hash_ms":3,"backup_ms":1,"swap_ms":5,"io_bytes_read":4096,"io_bytes_written":8192,"timers":{"fsync_ms":1}},
  "error_id": "E_EXDEV",
  "exit_code": 50,
  "error": {"kind":"io","errno":18,"message":"EXDEV: cross-device link","remediation":"enable degraded fallback or align mounts"},
  "redaction": {"applied": false, "rules": []},
  "seq": 42
}
```

---

## Implementation notes (code & schema)

- Schema: update `SPEC/audit_event.v2.schema.json` with new optional properties/defs; keep existing constraints untouched.
- Code: add envelope helpers and optional feature‑gated environment metadata.
- Perf/IO: wire additional fields only where cheap/available; do not regress critical paths.

---

## Acceptance checks (v2.1)

- Tests validate new examples against the updated v2 schema.
- Events include `event_id` and `run_id` when available.
- No changes to existing required fields; v2 clients continue to pass.
- Optional perf/IO and resource fields appear where instrumentation exists.
- Added stages (e.g., `preflight.summary`) validate against schema and are emitted by orchestrators.

---

## Non‑goals for v2.1

- Breaking changes to existing fields or requirements.
- Mandatory environment/host/actor metadata (keep optional for privacy/security).
- Removing `path` in favor of `resource` (co‑exist in v2.x).

---

## Follow‑ups

- Consider per‑stage separate schema files in v3 (typed unions) for stronger validation.
- Consider streaming signatures/attestation of event sequences (Merkle chain) as a future security enhancement.
