# Features UX Refactor — Proposal

Goal: Make Switchyard’s capabilities easy to understand, adopt, and configure. Organize features into coherent families, provide ergonomic builders/profiles, and gate optional pieces via Cargo features. Minimize cognitive load while keeping expert control.

Outcomes

- A single, consumer-centric map of what Switchyard offers.
- Clear defaults that “just work” in dev, CI, and prod.
- Optional capabilities behind explicit Cargo features.
- Stable, curated public namespaces and a prelude.

---

## 1) Feature Families (consumer view)

Think in families rather than an undifferentiated list. Each family has defaults and optional knobs.

- Core Safety & Determinism (always on)
  - SafePath, TOCTOU-safe syscalls, deterministic UUIDv5 IDs, dry-run parity/redaction, atomic symlink ops, backup/restore, prune.
  - Public surface: `switchyard::prelude::*`.

- Policy & Gating
  - Preflight evaluator, preservation support, ownership checks, risk gates (SUID/SGID, hardlinks), scope (allow_roots/forbid_paths), extra mount checks.
  - Profiles: `production`, `coreutils_switch`, `permissive_dev`.

- Observability & Audit
  - Stage-typed audit events with envelope/redaction; sinks and test utilities; optional perf/attestation blocks.

- Concurrency & Workflow
  - Locking (bounded wait with metrics) and smoke tests (opt-in) with auto-rollback.

- Extensibility Adapters
  - LockManager, OwnershipOracle, Attestor, SmokeTestRunner, Path helpers.

---

## 2) Cargo Features (capability view)

Introduce explicit, opt-in features for optional capabilities. Keep `default = ["prelude"]` to maximize DX.

```toml
[features]
# On by default for DX
prelude = []

# Optional capabilities
jsonl-file-sink = []            # rename of current `file-logging`
config = ["serde", "serde_yaml"]
serde-reports = ["serde"]      # reports and IDs derive Serialize/Deserialize
attestation = []                # enable attestation adapters and schema fields
smoke-tests = []                # enable default SmokeTestRunner helpers
tracing = ["tracing"]          # opt-in spans at API boundaries
test-utils = []                 # TestEmitter/TestAudit, macros
```

Acceptance:

- `cargo build` with defaults yields a minimal but usable library (`prelude` on).
- Optional flags compile independently and together (feature matrix CI job).

---

## 3) Public API Grouping (namespace view)

Provide a curated prelude and stable re-exports for discoverability. Keep low-level atoms internal.

- `switchyard::prelude::*`
  - `Switchyard`, `ApiBuilder`, `Policy`, `Plan`, `PlanInput`, `ApplyMode`, `FileLockManager`, `FsOwnershipOracle`.

- `switchyard::api` — builders and stages
  - `ApiBuilder`, `Switchyard`, `errors`, `plan`, `preflight`, `apply`, `rollback`.

- `switchyard::policy` — types, profiles, gating
  - `types`, `profiles`, `gating` (single evaluator entrypoint), `builder`.

- `switchyard::types` — plans, reports, ids, errors, safepath

- `switchyard::logging` — facade and sinks
  - `StageLogger`, `EventBuilder`, `FactsEmitter`, `AuditSink`, `JsonlSink` (or `FileJsonlSink` behind `jsonl-file-sink`).

- `switchyard::adapters` — curated adapters
  - `lock::{FileLockManager,..}`, `ownership::{FsOwnershipOracle,..}`, `attest::*` (behind `attestation`), `smoke::*` (behind `smoke-tests`).

Acceptance:

- No public re-export of `open_dir_nofollow|atomic_symlink_swap|fsync_parent_dir`.
- Prelude compiles examples from README verbatim.

---

## 4) Documentation (UX view)

Convert the README “Features” list into a user journey with three quickstarts and one matrix.

- Quickstarts (copy-pasteable):
  1) Dev sandbox (no locks): prelude + permissive_dev profile + JsonlSink.
  2) CI gated apply: production profile + locking (file) + smoke-tests.
  3) Airgapped prod: production profile + no attestation features (omit feature).

- Feature Matrix
  - Rows: Families; Columns: Dev, CI, Prod; Cells: enabled features, required adapters.

- Schema version guidance
  - Brief v1→v2 audit schema differences; link to SPEC.

Acceptance:

- README shows code that compiles under `--all-features` and under defaults.

---

## 5) Policy Profiles and Builders (DX view)

- `Policy::production()`, `coreutils_switch()`, `permissive_dev()` under `policy::profiles`.
- `PolicyBuilder` for custom tweaks with nested builders: `.scope()`, `.risks()`, `.apply()`, `.governance()`.
- `ApiBuilder` for assembling `Switchyard` with timeouts and adapters.

Acceptance:

- Examples compile and demonstrate concise construction flows.

---

## 6) Audit Schema v2 alignment (Obs view)

- Adopt `schema_version=2` (see `zrefactor/audit_event_schema_overhaul.PROPOSAL.md`) with stage conditionals.
- Enforce `error_id`, `summary_error_ids`, `perf`, `lock_backend/attempts` where appropriate.

Acceptance:

- Schema tests validate representative events across stages.

---

## 7) Migration & Backcompat

- Keep current capabilities working under new namespaces.
- Add new Cargo features without breaking existing users (defaults unchanged except adding `prelude`).
- Dual-write audit v1/v2 for one release (behind an env or feature) to ease downstream migration.

---

## 8) CI Guardrails

- Feature matrix build: all combinations of optional features (within reason).
- Grep gates from cohesion report: forbid ad-hoc gating outside policy, forbid direct FactsEmitter usage outside logging, no `#[path]` in `src/api/**`.

---

## 9) Stepwise Plan

- PR1: Prelude + curated re-exports; README quickstarts + matrix.
- PR2: Cargo features (`jsonl-file-sink`, `serde-reports`, `test-utils`, `config`, `smoke-tests`, `attestation`, `tracing`).
- PR3: Policy profiles and PolicyBuilder.
- PR4: ApiBuilder and PlanBuilder DSL (see `library_consumer_dx.INSTRUCTIONS.md`).
- PR5: Logging facade + audit schema v2 + tests.
- PR6: Finalize guardrails and remove deprecated shims.

---

## References

- Existing capabilities: `zrefactor/FEATURES_CATALOG.md`
- Consumer DX plan: `zrefactor/library_consumer_dx.INSTRUCTIONS.md`
- Cohesion and guardrails: `zrefactor/responsibility_cohesion_report.md`
- Audit schema v2: `zrefactor/audit_event_schema_overhaul.PROPOSAL.md`
