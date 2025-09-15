# Switchyard v1 Documentation Plan (Public, Library-Focused)

Note: This plan targets production operators/SREs and Rust integrators on Debian/Ubuntu servers. It documents the Switchyard library crate (not the CLI). Every technical claim below is backed by in-repo sources and inline citations.

## 1) Goals & Non-Goals

- [x] Safety-first operator guidance for using the library in production (SafePath, locking, rollback, deterministic behavior) (see `cargo/switchyard/src/lib.rs`:4-10)
- [x] Clear, deterministic behavior documentation including plan/action IDs, dry-run redaction, and stage parity (see `cargo/switchyard/src/types/ids.rs`; `cargo/switchyard/src/logging/redact.rs`)
- [x] Complete rollback procedures with evidence of reverse-order rollback and idempotence (see `cargo/switchyard/src/api/apply/mod.rs`; `cargo/switchyard/src/fs/restore/`)
- [x] Audit trail and facts schema usage with provenance and exit codes mapping (see `cargo/switchyard/src/logging/audit.rs`; `cargo/switchyard/SPEC/SPEC.md`)
- [x] Locking model, timeouts, and error mapping to E_LOCKING with lock_wait_ms fact (see `cargo/switchyard/src/api/apply/mod.rs`; `cargo/switchyard/tests/locking_required.rs`)

Non-Goals

- [ ] CLI manual or flags reference (out of scope; this is the library)
- [ ] Migration/backwards-compat guides (v1 baseline only)
- [ ] Internal/private development notes (public docs only)

What we’ll cite

- `cargo/switchyard/src/lib.rs`, `api/*`, `fs/*`, `types/*`, `logging/*`, `policy/*`, tests under `cargo/switchyard/tests/*`, SPEC under `cargo/switchyard/SPEC/*`.

## 2) Audience & Tone

- Primary: Debian/Ubuntu production administrators and SREs integrating Switchyard-backed changes in controlled environments.
- Secondary: Rust library integrators embedding Switchyard; security reviewers verifying guarantees against SPEC.
- Tone: Safety-first, procedural, evidence-anchored. Use checklists, minimal imperative examples, and stable terminology. Prefer conservative defaults and explicit opt-ins.

What we’ll cite

- `cargo/switchyard/README.md` overview and examples; crate docs in sources.

## 3) Source of Truth & Terminology

- Normative source: `cargo/switchyard/SPEC/SPEC.md` (guarantees, schemas, error codes) (see `SPEC.md` sections 1–6: `cargo/switchyard/SPEC/SPEC.md`:10-92,94-229,234-252)
- Key terms (as used in code/spec):
  - Plan / Action: Structured steps (`EnsureSymlink`, `RestoreFromBackup`) built from `PlanInput` (see `cargo/switchyard/src/types/plan.rs`:26-41,33-36)
  - Preflight: Policy gating and probes; emits per-action rows and a summary (see `cargo/switchyard/src/api/preflight/mod.rs`)
  - Apply: Executes actions with atomic swap + backups, emits facts, enforces locking, optional smoke + attestation, reverse-order rollback (see `cargo/switchyard/src/api/apply/mod.rs`)
  - Rollback: Automatic reverse-order restore on failure; idempotent (see `cargo/switchyard/src/api/apply/mod.rs`; `cargo/switchyard/src/fs/restore/`)
  - SafePath: Root-anchored, `..`-rejecting typed path for all mutations (see `cargo/switchyard/src/types/safepath.rs`:11-27,28-47,60-67)
  - LockManager: Adapter enforcing single mutator with bounded wait (see `cargo/switchyard/src/adapters/lock/mod.rs`:4-8; `cargo/switchyard/src/adapters/lock/file.rs`:34-61)
  - EXDEV degraded mode: Policy-controlled fallback for cross-filesystem swap (see `cargo/switchyard/src/fs/atomic.rs`:86-95; `cargo/switchyard/src/api/apply/handlers.rs`:101-115)
  - Audit Facts: Facts schema v2 with envelope fields (`schema_version`, `ts`, `plan_id`, `run_id`, `event_id`), error ids/exit codes, optional attestation (see `cargo/switchyard/src/logging/audit.rs`; `cargo/switchyard/SPEC/audit_event.v2.schema.json`)

What we’ll cite

- `SPEC.md`, `types/plan.rs`, `api/*`, `types/safepath.rs`, `adapters/lock/*`, `logging/audit.rs`.

## 4) Inventory (Repo Recon)

Crate/Package

| Item | Value |
|---|---|
| Crate name | `switchyard` (see `cargo/switchyard/Cargo.toml`:1-11)
| Version | 0.1.0 (see `cargo/switchyard/Cargo.toml`:1-7)
| Edition | 2021 (see `cargo/switchyard/Cargo.toml`:1-7)
| Features | `file-logging` for file-backed JSONL sinks (see `cargo/switchyard/Cargo.toml`:28-36; `src/logging/facts.rs`:23-31,52-75,77-91)

Modules and Public Items (major, grouped)

- `switchyard::api` (see `cargo/switchyard/src/api/mod.rs`)
  - `struct Switchyard<E: FactsEmitter, A: AuditSink>` with builder methods (`with_lock_manager`, `with_ownership_oracle`, `with_attestor`, `with_smoke_runner`, `with_lock_timeout_ms`) and API surface `plan`, `preflight`, `apply`, `plan_rollback_of`, `prune_backups` (doc: partial Y; doctest: N)
  - Submodules:
    - `api/apply/mod.rs` (`run`) — apply engine (doc: Y; doctest: N)
    - `api/apply/handlers.rs` — per-action handlers; before/after hashes; degraded flags (doc: Y; doctest: N)
    - `api/apply/audit_fields.rs` — `insert_hashes`, `maybe_warn_fsync` (doc: Y; doctest: N)
    - `api/preflight/mod.rs` (`run`) — preflight stage, rows emission (doc: Y; doctest: N)
    - `api/preflight/row_emitter.rs` — row assembly and fact emission (doc: partial Y; doctest: N)
    - `api/plan.rs` — plan builder (delegated) (doc: N; doctest: N)
    - `api/errors.rs` — `ApiError`, `ErrorId`, mapping to exit codes (doc: Y; doctest: N)

- `switchyard::types` (see `cargo/switchyard/src/types/mod.rs`:1-12)
  - `ApplyMode`, `PlanInput`, `Plan`, `Action` (doc: N; doctest: N) (`cargo/switchyard/src/types/plan.rs`:1-41)
  - `PreflightReport`, `ApplyReport`, `PruneResult` (doc: Y; doctest: N) (`cargo/switchyard/src/types/report.rs`:4-22,24-30)
  - `SafePath` with `from_rooted`, `as_path`, `rel` (doc: Y; doctest: Y via unit tests) (`cargo/switchyard/src/types/safepath.rs`:11-27,60-67,69-105)
  - `ids::{plan_id, action_id}` deterministic UUIDv5 (doc: Y; doctest: N) (`cargo/switchyard/src/types/ids.rs`:7-13,31-46)
  - `errors::{ErrorKind, Error, Result}` (doc: Y; doctest: N) (`cargo/switchyard/src/types/errors.rs`:1-21)

- `switchyard::fs` (see `cargo/switchyard/src/fs/mod.rs`:1-15,16-26)
  - `atomic::{open_dir_nofollow, atomic_symlink_swap, fsync_parent_dir}` (doc: Y; doctest: N) (`cargo/switchyard/src/fs/atomic.rs`:1-8,22-33,72-97)
  - `swap::replace_file_with_symlink` (backs up → atomic swap) (doc: Y; doctest: Y via unit tests) (`cargo/switchyard/src/fs/swap.rs`:11-22,82-133,135-209)
  - `backup::{index::backup_path_with_tag, snapshot::create_snapshot, index::has_backup_artifacts, sidecar::{...}, prune::prune_backups}` + sidecar schema v1/v2 (hash) (doc: Y; doctest: Y via unit tests) (`cargo/switchyard/src/fs/backup/{index.rs,snapshot.rs,sidecar.rs,prune.rs}`)
  - `restore::{engine, steps, idempotence, selector, types}` including `restore_file` and integrity checks (doc: Y; doctest: Y via unit tests) (`cargo/switchyard/src/fs/restore/`)
  - `meta::{sha256_hex_of, resolve_symlink_target, kind_of, detect_preservation_capabilities}` (doc: Y) (`cargo/switchyard/src/fs/meta.rs`:19-26,28-44,46-63,65-106)
  - `mount::{ensure_rw_exec, ProcStatfsInspector}` (doc: Y; doctest: Y via unit tests) (`cargo/switchyard/src/fs/mount.rs`:69-80,82-132)

- `switchyard::logging` (see `cargo/switchyard/src/logging/mod.rs`:1-7)
  - `facts::{FactsEmitter, AuditSink, JsonlSink}` (doc: Y; doctest: N; feature `file-logging` adds FileJsonlSink) (`cargo/switchyard/src/logging/facts.rs`:4-21,23-31,52-75)
  - `audit::{StageLogger, ensure_provenance, SCHEMA_VERSION}` (doc: Y) (`cargo/switchyard/src/logging/audit.rs`)
  - `redact::{redact_event, ts_for_mode, TS_ZERO}` (doc: Y; tests present) (`cargo/switchyard/src/logging/redact.rs`:54-63,64-105)

- `switchyard::adapters`
  - `lock::{LockManager, LockGuard, FileLockManager}` (doc: Y; doctest: Y via tests) (`cargo/switchyard/src/adapters/lock/mod.rs`:4-8; `cargo/switchyard/src/adapters/lock/file.rs`:34-61,63-97)
  - `ownership::{OwnershipOracle, FsOwnershipOracle}` (doc: Y) (`cargo/switchyard/src/adapters/ownership/mod.rs`:4-13; `fs.rs`:10-24)
  - `path::PathResolver` (doc: N) (`cargo/switchyard/src/adapters/path.rs`:1-5)
  - `smoke::{SmokeTestRunner, DefaultSmokeRunner}` (doc: Y) (`cargo/switchyard/src/adapters/smoke.rs`:15-20,22-65)
  - `attest::{Attestor, Signature}` (doc: Y) (`cargo/switchyard/src/adapters/attest.rs`:3-14)

- `switchyard::policy`
  - `Policy` configuration and presets (`production_preset`, `coreutils_switch_preset`); gating and rescue helpers (doc: Y; doctest on examples) (`cargo/switchyard/src/policy/config.rs`:141-170,181-240,242-273)
  - `gating` helper used in apply-stage parity (see `cargo/switchyard/src/api/apply/mod.rs`:182-224)
  - `rescue::{verify_rescue_*}` (doc: Y; tests present) (`cargo/switchyard/src/policy/rescue.rs`:25-41,86-105,112-135)

What we’ll cite

- All paths listed per bullet; line ranges included for key items above.

## 5) Guarantees & Safety Model (Evidence-Backed)

- Atomic swap and TOCTOU-safe sequence: open parent with `O_DIRECTORY|O_NOFOLLOW`, `symlinkat` on tmp, `renameat` tmp→final, `fsync(parent)` (see `cargo/switchyard/src/fs/atomic.rs`:22-33,63-71,72-85). SPEC reiterates normative sequence (see `cargo/switchyard/SPEC/SPEC.md`).
- Deterministic IDs/ordering: UUIDv5 `plan_id` over serialized actions; `action_id` = v5(plan_id, action+index) (see `cargo/switchyard/src/types/ids.rs`:7-13,31-46). SPEC Determinism (see `cargo/switchyard/SPEC/SPEC.md`:70-74,15-16).
- Preflight gates, preservation/probes: rw+exec mounts, immutability, hardlink, suid/sgid risk, source trust, ownership, allow_roots/forbid_paths, preservation capability map (see `cargo/switchyard/src/api/preflight/mod.rs`:51-168,170-201,256-281; `cargo/switchyard/src/preflight/checks.rs`:5-16,18-31,33-58,60-90,92-121; `cargo/switchyard/src/fs/meta.rs`:65-106). SPEC Preflight schema (see `cargo/switchyard/SPEC/SPEC.md`:129-160).
- Apply behavior, fail-closed decisions: gating parity; without lock manager in Commit and require_lock_manager → E_LOCKING; per-action attempt/result facts; degraded EXDEV handling with telemetry (see `cargo/switchyard/src/api/apply/mod.rs`:70-150,172-181,182-224,371-433; `cargo/switchyard/src/api/apply/handlers.rs`:63-99,101-115).
- Reverse-order rollback and idempotence: on first failure, restore executed actions in reverse; smoke failure triggers auto-rollback unless disabled (see `cargo/switchyard/src/api/apply/mod.rs`:249-299,301-369). Restore idempotence paths (see `cargo/switchyard/src/fs/restore/*`).
- Locking requirements and timeouts: `LockManager` required in production; bounded wait → E_LOCKING; `lock_wait_ms` recorded (see `cargo/switchyard/src/api/apply/mod.rs`:70-117,172-180; tests `cargo/switchyard/tests/locking_required.rs`:44-61).
- Rescue/fallback expectations: verify BusyBox or GNU subset with exec bits; policy gates require rescue in Commit (see `cargo/switchyard/src/policy/rescue.rs`:11-21,25-41,46-84; `cargo/switchyard/src/api/preflight/mod.rs`:39-46,293-317 summary). SPEC §6 Rescue (see `cargo/switchyard/SPEC/SPEC.md`:64-69).
- Audit facts schema v2, redaction, provenance, attestation: envelope fields, redaction in dry-run, before/after hashes, attestation bundle on success (see `cargo/switchyard/src/logging/audit.rs`; `cargo/switchyard/src/logging/redact.rs`; `cargo/switchyard/src/api/apply/handlers.rs`; SPEC schema `cargo/switchyard/SPEC/audit_event.v2.schema.json`).
- Backups and integrity: sidecar `backup_meta.v1/v2` with `prior_kind`, `mode`, optional `payload_hash` (sha256); restore verifies hash when present; durability fsync of parent (see `cargo/switchyard/src/fs/backup/{index.rs,snapshot.rs,sidecar.rs}`; `cargo/switchyard/src/fs/restore/{engine.rs,steps.rs,integrity.rs}`).
- Cross-filesystem support (EXDEV): degraded fallback allowed by policy sets `degraded=true`; otherwise mapped to `E_EXDEV` with reason (see `cargo/switchyard/src/fs/atomic.rs`:86-95; `cargo/switchyard/src/api/apply/handlers.rs`:63-71,83-96,101-107). SPEC Filesystems (see `cargo/switchyard/SPEC/SPEC.md`:86-91,304-317).

What we’ll cite

- `fs/atomic.rs`, `fs/swap.rs`, `fs/backup/*`, `fs/restore/*`, `api/apply/*`, `api/preflight/*`, `logging/*`, `types/ids.rs`, SPEC sections 1–6, 10–11.

## 6) Documentation Artifacts to Produce

- Crate-level rustdoc overview + minimal examples
  - Purpose: One-stop safety + determinism summary and how to compose adapters.
  - Audience: Rust integrators, security reviewers.
  - Acceptance: Example compiles; links to `Policy::production_preset` and `Switchyard::with_*` (see `cargo/switchyard/src/api/mod.rs`; `cargo/switchyard/src/policy/config.rs`:141-170).

- API item coverage
  - Document every `pub` item in `api`, `types`, `logging`, `fs`, `adapters`, `policy`.
  - Acceptance: `#![deny(missing_docs)]` passes; at least one doctest for each major type or function (e.g., SafePath, Policy presets).

- Guide site (mdBook)
  - Chapters per §7 below. Debian-first pragmatic steps.
  - Acceptance: mdbook builds; linkcheck clean; examples compile.

- Runnable examples under `cargo/switchyard/examples/`
  - `01_dry_run.rs` (plan/preflight/apply DryRun)
  - `02_commit_with_lock.rs` (FileLockManager; Commit)
  - `03_rollback.rs` (force failure → verify rollback)
  - `04_audit_and_redaction.rs` (facts capture + redaction parity)
  - `05_exdev_degraded.rs` (set `allow_degraded_fs=true`, observe `degraded=true`)
  - Acceptance: `cargo run --example <name>` on Debian/Ubuntu (non-root acceptable except operations that require permissions under temp roots).

- Reference pages
  - Error/exit codes mapped to `ErrorId` and SPEC TOML (see `cargo/switchyard/SPEC/error_codes.toml`:1-12; `cargo/switchyard/src/api/errors.rs`:31-73)
  - Preflight schema reference mapped to SPEC §4 and `preflight/yaml.rs` exporter (see `cargo/switchyard/src/preflight/yaml.rs`:1-34)
  - Audit event schema v2 (`cargo/switchyard/SPEC/audit_event.v2.schema.json`)
  - Policy knobs (selected `Policy` fields with security rationale) (see `cargo/switchyard/src/policy/config.rs`:15-108)

- Operator checklists
  - Preflight, Commit/apply, Rollback, Incident recovery with manual steps for sidecar-backed restore (see `cargo/switchyard/README.md`:211-270)

What we’ll cite

- Code paths above; SPEC schemas; README manual restore steps.

## 7) Proposed Guide (mdBook) Outline (Debian-First)

- Introduction — What Switchyard Is (library, guarantees)
  - Objective: Scope and guarantees overview; not a CLI.
  - Sources: `cargo/switchyard/SPEC/SPEC.md`:10-23; `cargo/switchyard/README.md`:3-14

- Quickstart (Debian)
  - Objective: plan → preflight → dry-run → commit with locking → smoke check → rollback on failure → capture facts.
  - Sources: `cargo/switchyard/src/api/mod.rs`; `policy/config.rs`:141-170; `adapters/lock/file.rs`:34-61; `api/apply/mod.rs`:70-117,301-369; `logging/redact.rs`:54-63

- Core Concepts
  - Plan/Actions/IDs — `PlanInput`, `Plan`, `Action`, UUIDv5 (sources: `types/plan.rs`:26-41; `types/ids.rs`:31-46)
  - Preflight — gating and rows (sources: `api/preflight/mod.rs`:51-168,256-281; SPEC §4)
    - Apply — atomic swap + backup/restore (sources: `fs/swap.rs`; `fs/backup/*`; `fs/restore/*`)
  - Rollback — reverse-order and idempotence (sources: `api/apply/mod.rs`; `fs/restore/*`)
  - Locking — bounded wait, E_LOCKING (sources: `api/apply/mod.rs`:70-117; test `tests/locking_required.rs`)
  - Rescue — BusyBox/GNU subset (sources: `policy/rescue.rs`:25-41,46-84)
  - EXDEV degraded mode — policy and telemetry (sources: `fs/atomic.rs`:86-95; `api/apply/handlers.rs`:101-107)
  - Audit Facts — schema, redaction, attestation (sources: `logging/audit.rs`:44-66,143-171; `logging/redact.rs`:64-105; SPEC §5)

- How-Tos
  - Configure Lock Manager (FileLockManager) (sources: `adapters/lock/file.rs`:34-61)
  - SafePath policies and path rooting (sources: `types/safepath.rs`:11-27,60-67)
  - Audit capture and verification (redaction parity) (sources: `logging/redact.rs`:64-105; tests `tests/locking_required.rs` for stage/ids)

- Reference
  - Public API map (points to rustdoc) (sources: `api/mod.rs`; `types/mod.rs`:7-12)
  - Exit codes table (sources: `SPEC/error_codes.toml`:1-12; `api/errors.rs`:61-73)
  - Preflight schema (SPEC §4; `preflight/yaml.rs`)
  - Audit event schema (SPEC §5)
  - Operational bounds (if applicable) (sources: SPEC §9 `cargo/switchyard/SPEC/SPEC.md`:297-303)

- Troubleshooting
  - Lock timeout (sources: `api/apply/mod.rs`:70-117; tests `tests/locking_timeout.rs`)
  - EXDEV disallowed (sources: `api/apply/handlers.rs`:63-71,83-96)
  - Smoke failures and auto-rollback (sources: `api/apply/mod.rs`:301-369)
  - Partial restoration (sources: `fs/restore/*`)

What we’ll cite

- Sections above with relevant paths/lines.

## 8) Quality Gates & Tooling

- Crate lints
  - Target: `#![deny(missing_docs, rustdoc::broken_intra_doc_links)]` and existing `#![forbid(unsafe_code)]`, clippy pedantic (see `cargo/switchyard/src/lib.rs`:1-3)

- Doctests
  - Target: `cargo test --doc -p switchyard` must pass.

- Rustdoc build
  - `cargo doc -p switchyard --no-deps` (and with `--all-features` to validate `file-logging` doc items)

- Book build
  - `mdbook build` under `cargo/switchyard/book/` (new) with `mdbook-linkcheck`, `mdbook-mermaid`

- Consistency checks
  - Ensure language for dry-run vs commit facts matches SPEC redaction guarantees (see `logging/redact.rs`:64-105; `SPEC/SPEC.md`:70-74,166-229)

What we’ll cite

- `src/lib.rs`, `logging/redact.rs`, SPEC §5, §7.

## 9) Work Plan (Tasks, Owners TBD)

Small (S)

- [x] Add crate-level rustdoc overview with Quickstart snippet (acceptance: compiles).
- [x] Document `SafePath` with examples; add doctest (acceptance: doctest passes) (`types/safepath.rs`).
- [x] Add reference page for `ErrorId`→exit codes (acceptance: matches `SPEC/error_codes.toml`).
- [x] Add example `01_dry_run.rs` using temp root and `SafePath`.

Medium (M)

- [ ] Generate API surface table from rustdoc JSON and reconcile with this inventory (acceptance: table checked into docs with symbol paths).
- [x] Draft mdBook Quickstart + Concepts chapters with citations (acceptance: mdbook build + linkcheck pass).
- [x] Write Reference: Preflight schema and Audit schema chapters (acceptance: matches SPEC exactly).

Large (L)

- [x] Full mdBook first pass (all chapters in §7) including Debian-first examples (acceptance: book builds; code snippets compile or `no_run`).
- [x] Troubleshooting section with verified snippets and references to tests (acceptance: steps reproducible on Debian/Ubuntu in temp roots).

## 10) Risks, Gaps, Open Questions

- [OPEN QUESTION] EXDEV degraded fallback semantics vs SPEC wording “safe copy + fsync + rename” (SPEC §2.10 REQ-F1) vs current symlink fallback using unlink+`symlinkat` (see `cargo/switchyard/SPEC/SPEC.md`:88-91; `cargo/switchyard/src/fs/atomic.rs`:86-95). Clarify if symlink replacement is exempt from copy+rename language.
- [OPEN QUESTION] Thread-safety “Send + Sync” statement in SPEC §14 vs trait bounds in logging adapters (FactsEmitter/AuditSink don’t require Sync) (see `cargo/switchyard/SPEC/SPEC.md`:350-353; `cargo/switchyard/src/logging/facts.rs`:4-10). Confirm scope and whether top-level `Switchyard<E,A>` is `Send + Sync` under typical `E/A`.
- [OPEN QUESTION] Minimal smoke suite exact command set in SPEC §11—default `DefaultSmokeRunner` validates symlink topology only (see `cargo/switchyard/src/adapters/smoke.rs`:28-64; `cargo/switchyard/SPEC/SPEC.md`:318-336). Align docs on production expectations.
- [OPEN QUESTION] Provenance completeness: SPEC mentions origin/helper/uid/gid/pkg; current `ensure_provenance` ensures `env_sanitized` only by default (see `cargo/switchyard/src/logging/audit.rs`:209-220). Clarify adapter responsibilities in docs.

Where we looked: files cited above and SPEC sections.

## 11) Timeline & Milestones (Proposed)

- Milestone 1 (Week 1): API coverage ≥ 90% documented; `cargo test --doc` passes; example `01_dry_run.rs` runs.
- Milestone 2 (Week 2): mdBook scaffold complete; Quickstart and Concepts drafted; linkcheck clean.
- Milestone 3 (Week 3): Reference + Troubleshooting finalized; examples `02_commit_with_lock.rs`, `03_rollback.rs`, `04_audit_and_redaction.rs`, `05_exdev_degraded.rs` compile.
- Final (Week 4): Full review against acceptance criteria; SPEC citations audited.

## 12) Acceptance Criteria (Definition of Done)

- [ ] Book builds clean; diagrams render; linkcheck passes.
- [ ] `cargo doc -p switchyard` builds with no missing docs; `cargo test --doc -p switchyard` passes.
- [ ] Every book claim has a nearby citation (file path + line range) to source or SPEC.
- [ ] Each core API (`Switchyard`, `Plan/Action`, `SafePath`, `Policy`, `LockManager`) has at least one example/doctest (compiles; `no_run` allowed).
- [ ] Reference pages match source artifacts verbatim (schemas/exit codes).
- [ ] Operator checklists exist for preflight, commit/apply, rollback, incident recovery (manual restore) (see `cargo/switchyard/README.md`:211-270).

---

Appendix — Test Artifacts to Reference (not user-facing docs, but for writer verification)

- Locking behavior and facts: `cargo/switchyard/tests/locking_required.rs`, `locking_timeout.rs`, `locking_stage_parity.rs`.
- Preflight YAML and golden: `cargo/switchyard/tests/preflight_yaml.rs`, `preflight_yaml_golden.rs`.
- Rollback and errors: `error_atomic_swap.rs`, `error_policy.rs`, `error_exdev.rs`, `error_backup_missing.rs`, `error_restore_failed.rs`, `smoke_required.rs`, `smoke_rollback.rs`.
- Attestation presence on success: `attestation_apply_success.rs`.
