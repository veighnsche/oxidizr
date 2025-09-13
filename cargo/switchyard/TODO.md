# Audit Event v2.1 — Implementation TODO

This checklist tracks all tasks required to implement and roll out audit event schema v2 in Switchyard.

Related docs:

- Proposal: `zrefactor/audit_event_schema_overhaul.PROPOSAL.md`
- Schema: `SPEC/audit_event.v2.schema.json`
- Current schema v1: `SPEC/audit_event.schema.json`
- Logging code: `src/logging/audit.rs`

---

## 1) Schema and Envelope (v2.1)

- [x] Add JSON Schema file `SPEC/audit_event.v2.schema.json` (envelope + stage-specific constraints)
- [ ] Switch default schema version in code
  - [ ] Edit `src/logging/audit.rs`: set `pub(crate) const SCHEMA_VERSION: i64 = 2;`
- [ ] Make envelope injection schema-aware and add v2.1 envelope fields
  - [ ] Edit `src/logging/audit.rs::redact_and_emit(...)`
    - [ ] Replace unconditional `obj.entry("path").or_insert(json!(""));` with:
      - [ ] For v2: do NOT inject `path` at all; keep whatever fields caller provided
      - [ ] For v1: preserve current behavior (inject empty string when absent)
    - [ ] Ensure `dry_run` and `redacted` are set from `AuditCtx.mode` (already present; keep intact)
  - [ ] Inject optional envelope fields (all best-effort, behind a small helper):
    - [ ] `event_id` (UUIDv7) if not present — add `fn new_event_id()` in `logging/audit.rs`
    - [ ] `run_id` — add to `AuditCtx` or compute once per `Switchyard` call; plumb from orchestrator entry
    - [ ] `switchyard_version` — read from `CARGO_PKG_VERSION`
    - [ ] Optional `build { git_sha, rustc }` — behind feature `envmeta` (read env vars / rustc_version)
    - [ ] Optional `host { hostname, os, kernel, arch }` — behind `envmeta`
    - [ ] Optional `process { pid, ppid }` — behind `envmeta`
    - [ ] Optional `actor { euid, egid }` — behind `envmeta`
    - [ ] Optional `redaction { applied, rules }` — derive from `AuditMode` and redaction policy
    - [ ] Optional `seq` — monotonic counter per `plan_id`/`run_id` (store in `AuditCtx` and increment per emit)
// Back-compat removed pre-v1: v2 is the only path; no env toggles or dual-write

- [ ] Schema refinements (align with emitted fields)
  - [ ] Edit `SPEC/audit_event.v2.schema.json` → add optional properties:
    - [ ] `before_kind` (string), `after_kind` (string) — used in `api/apply/handlers.rs`
    - [ ] `degraded_reason` (string) — used when EXDEV fallback in `handle_ensure_symlink`
    - [ ] `sidecar_integrity_verified` (boolean|null) — used in `handle_restore`
    - [ ] `backup_durable` (boolean) — recorded from policy in apply events
  - [ ] Preflight summary handling:
    - [ ] Option A (preferred): introduce `preflight.summary` stage
      - [ ] Edit `src/logging/audit.rs::Stage` to add `PreflightSummary` → `as_event() => "preflight.summary"`
      - [ ] Edit `src/logging/mod.rs` re-export if necessary
      - [ ] Edit `src/api/preflight/mod.rs`: emit summary using `slog.preflight_summary()`
      - [ ] Edit `SPEC/audit_event.v2.schema.json`: add `"preflight.summary"` to `stage` enum and remove preflight hard requirement for `path/current_kind/planned_kind` when stage is summary
    - [ ] Option B: keep single `preflight` stage but gate requirements by `action_id`
      - [ ] Edit `SPEC/audit_event.v2.schema.json`: for `stage==preflight` require `path/current_kind/planned_kind` only when `action_id` is present
    - [ ] Choose one option and implement consistently in code and schema

## 2) Stage-specific Emissions (API call sites)

- [ ] `apply.attempt` (success + failure)
  - [ ] File: `src/api/apply/mod.rs::run()` — success path
    - [ ] Ensure event includes `lock_backend`, `lock_attempts`, `lock_wait_ms` (already present at call site: `slog.apply_attempt().merge(json!({ ... }))`)
    - [ ] After v2 envelope change, confirm no `path` is injected (see §1)
  - [ ] File: `src/api/apply/lock.rs::acquire()` — failure path
    - [ ] Ensure failure event includes `lock_backend`, `lock_wait_ms`, `lock_attempts`, `error_id=E_LOCKING`, `exit_code`
    - [ ] Confirm no `path` injected for v2

- [ ] `apply.result` per-action
  - [ ] File: `src/api/apply/handlers.rs::handle_ensure_symlink`
    - [ ] Success: event already includes `before_kind/after_kind`, `degraded` and `duration_ms`
    - [ ] Hashes: uses `apply/audit_fields.rs::insert_hashes` — confirm both `before_hash` and `after_hash` set `hash_alg="sha256"`
    - [ ] Failure: ensure `error_id` + `exit_code` are set (present via `api/errors.rs` mapping)
    - [ ] v2.1: add optional `error { kind, errno, message, remediation }` (see §5)
  - [ ] File: `src/api/apply/handlers.rs::handle_restore`
    - [ ] Success/failure: ensure `before_kind/after_kind`; on failure set `error_id` + `exit_code`
    - [ ] Sidecar integrity: field `sidecar_integrity_verified` stays optional
    - [ ] v2.1: add optional `error { kind, errno, message, remediation }` on failure

- [ ] `apply.result` summary
  - [ ] File: `src/api/apply/mod.rs::run()` — final summary block
    - [ ] Ensure `fields` does NOT set `path` at all for v2
    - [ ] Include `perf` aggregate (hash_ms, backup_ms, swap_ms)
    - [ ] Optional `attestation` via `adapters::attest::build_attestation_fields`
    - [ ] On failure: set `error_id`, `exit_code`, and `summary_error_ids` (via `api/errors::infer_summary_error_ids`)
    - [ ] v2.1: include optional `resource` when available; propagate `event_id`, `run_id`, `seq`

- [ ] `preflight` per-action rows
  - [ ] File: `src/api/preflight/rows.rs::push_row_emit`
    - [ ] Emit includes: `.path(path)`, `current_kind`, `planned_kind` (already present)
    - [ ] Add `policy_ok` if present (already merged), `preservation`, `preservation_supported`, `provenance` best-effort
  - [ ] File: `src/api/preflight/mod.rs::run()` — summary
    - [ ] Keep `rescue_profile` and failure mapping to `E_POLICY`; may include `summary_error_ids`
    - [ ] v2.1: emit `preflight.summary` stage; attach `run_id`, `seq`, and `redaction` info

- [ ] `prune.result`
  - [ ] File: `src/api/mod.rs::prune_backups`
    - [ ] Ensure required: `path`, `pruned_count`, `retained_count` (already present)
    - [ ] Optional: `backup_tag`, `retention_count_limit`, `retention_age_limit_ms`

## 3) Tests and Golden Samples (v2.1)

- [ ] Add a v2 schema validation test (parallel to v1 test)
  - [ ] File: `tests/audit/audit_schema_v2.rs` (new)
    - [ ] Prepare API (DryRun) → collect events via a test `FactsEmitter`
    - [ ] Load `include_str!("../SPEC/audit_event.v2.schema.json")`
    - [ ] Validate every emitted event using `jsonschema::JSONSchema`
- [ ] Update/extend integration tests
  - [ ] File: `tests/locking/locking_stage_parity.rs`
    - [ ] Assert `apply.attempt` includes `lock_backend`/`lock_attempts` and no `path` under v2
  - [ ] File: `tests/audit/provenance_presence.rs`
    - [ ] Keep provenance present where expected; compatible with v2
  - [ ] File: `tests/audit/preflight_summary_error_id.rs`
    - [ ] Ensure summary mapping still includes `error_id=E_POLICY` on failure
  - [ ] File: `tests/audit/summary_error_ids_ownership.rs`
    - [ ] Ensure ownership-related stops emit `summary_error_ids` including `E_OWNERSHIP`
- [ ] Add tests for v2.1 additions
  - [ ] New: `tests/audit/envelope_v2_1.rs` — assert presence/shape of `event_id`, `run_id`, `redaction`, `seq`
  - [ ] New: `tests/audit/resource_v2_1.rs` — assert optional `resource` object shape when available
  - [ ] New: `tests/audit/error_object_v2_1.rs` — assert `error` object on representative failures
  - [ ] New: `tests/audit/perf_expanded_v2_1.rs` — assert `perf.io_bytes_*` and `perf.timers.*` when measured
  - [ ] New: `tests/audit/hashes_multi_alg_v2_1.rs` — if enabled, assert `hashes[]` structure
- [ ] Golden examples under `tests/golden/`
  - [ ] Add JSON samples for: `apply.attempt` success, `apply.result` per-action success, `apply.result` summary failure, `preflight` row, `prune.result`
  - [ ] Optional: add a script/test to compare redacted vs non-redacted for DryRun parity


## 4) CI and Rollout (v2.1 only)

- [ ] CI job: validate JSON lines against v2 schema
  - [ ] Add a small Rust tool or script to read emitted events and validate via `jsonschema`
  - [ ] Wire into `.github/workflows/ci.yml` (new job or extend existing)
- [ ] SPEC docs
  - [ ] Update `SPEC/audit_event.v2.schema.json` (done) with v2.1 optional fields
  - [ ] Add `SPEC/audit_event.v2.md` summarizing required fields per stage; formats; envelope flags; list v2.1 additions
  - [ ] Link from `SPEC/SPEC.md` and crate README

## 5) Documentation (update for v2.1)

- [ ] Update `cargo/switchyard/README.md`: add schema v2 section; reference env toggles
- [ ] Update `zrefactor/audit_event_schema_overhaul.PROPOSAL.md`: mark implemented items, keep examples in sync with code
- [ ] Update acceptance greps in zrefactor docs to reflect v2 fields (lock, perf, attestation, summary behavior)
- [ ] Add `SPEC/audit_event.v2.md` and wire into SPEC index (see §4)
  - [ ] Include examples with `preflight.summary`, envelope fields, `resource`, expanded `perf`, `error` object

## 6) Acceptance Criteria (v2.1)

- [ ] `cargo test -p switchyard` passes with schema v2
- [ ] Golden tests validate v2 events; schema validator test suite passes
- [ ] No forced empty `path` on summary events; `path` omitted or `null` where applicable
- [ ] CI includes schema validation

---

// Removed: no dual-write, no v1 mapping. v2 (with v2.1 additions) is the only supported schema pre-1.0.

---

## Notes / References

- Error taxonomy and exit codes: `api/errors.rs`, `SPEC/error_codes.toml`
- Logging facade: `src/logging/{audit.rs,mod.rs}`
- Apply orchestrator and handlers: `src/api/apply/{mod.rs,handlers.rs}`
- Preflight orchestrator: `src/api/preflight/mod.rs`
- Prune path: `src/api/mod.rs::prune_backups`
