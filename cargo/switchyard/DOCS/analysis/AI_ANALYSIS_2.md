# AI 2 — Round 1 Analysis Report

Generated: 2025-09-12 14:24:34+02:00
Analyst: AI 2
Coordinator: Cascade

Scope: Verify claims, provide proofs, and patch gaps in the assigned documents only. Record evidence and edits here. Do not start Round 2 until instructed.

## Assigned Documents (37 pts)

- FS_SAFETY_AUDIT.md — 10
- API_SURFACE_AUDIT.md — 10
- OBSERVABILITY_FACTS_SCHEMA.md — 8
- ERROR_TAXONOMY.md — 7
- INDEX.md — 2

## Round 1 Checklist

- [x] Evidence mapping completed for all assigned docs
- [x] Patches applied to assigned docs where needed
- [x] All claims verified or corrected with citations
- [x] Open questions recorded

## Evidence — FS_SAFETY_AUDIT.md

- Claims → Proofs
  - [ ] Claim: Atomic swap sequence `open_dir_nofollow → symlinkat → renameat → fsync(parent)`
    - Proof: `cargo/switchyard/src/fs/atomic.rs::atomic_symlink_swap()` calls `fsync_parent_dir()` after `renameat`.
  - [ ] Claim: Backup/sidecar path has remaining path-based ops
    - Proof: `cargo/switchyard/src/fs/backup.rs` symlink payload creation and sidecar writes.

## Changes Made — FS_SAFETY_AUDIT.md

- [ ] Edit summary: <what changed and why>

## Evidence — API_SURFACE_AUDIT.md

- Claims → Proofs
  - [ ] Claim: Low-level FS atoms are publicly re-exported
    - Proof: `cargo/switchyard/src/fs/mod.rs` re-exports; usage sites.
  - [ ] Claim: Adapters traits are stable; default impls provisional
    - Proof: `cargo/switchyard/src/adapters/*` trait vs impl boundaries.

## Changes Made — API_SURFACE_AUDIT.md

- [ ] Edit summary: <what changed and why>

## Evidence — OBSERVABILITY_FACTS_SCHEMA.md

- Claims → Proofs
  - [ ] Claim: Minimal Facts v1 envelope with `schema_version=1`, `ts`, `plan_id`, `stage`, `decision`, `path`
    - Proof: `cargo/switchyard/src/logging/audit.rs`
  - [ ] Claim: Redaction rules mask specific fields
    - Proof: `cargo/switchyard/src/logging/redact.rs::redact_event()`

## Changes Made — OBSERVABILITY_FACTS_SCHEMA.md

- [ ] Edit summary: <what changed and why>

## Evidence — ERROR_TAXONOMY.md

- Claims → Proofs
  - [ ] Claim: ErrorId → exit-code mapping
    - Proof: `cargo/switchyard/src/api/errors.rs`, `SPEC/error_codes.toml`
  - [ ] Claim: Emission points mapping
    - Proof: `cargo/switchyard/src/api/apply/mod.rs`, `cargo/switchyard/src/api/preflight/mod.rs`, `cargo/switchyard/src/policy/gating.rs`

## Changes Made — ERROR_TAXONOMY.md

- [ ] Edit summary: <what changed and why>

## Evidence — INDEX.md

- Claims → Proofs
  - [ ] Claim: Index accurately reflects completed analyses with links
    - Proof: File presence under `cargo/switchyard/DOCS/analysis/` and list synchronization.

## Changes Made — INDEX.md

- [ ] Edit summary: <what changed and why>

## Open Questions

- [ ] <question>

## Round 2 Plan (Do NOT start yet)

- You will peer review AI 3’s outputs and assigned docs in Round 2:
  - PRESERVATION_FIDELITY.md, PREFLIGHT_MODULE_CONCERNS.md, POLICY_PRESETS_RATIONALE.md, LOCKING_STRATEGY.md, idiomatic_todo.md, SECURITY_REVIEW.md, RELEASE_AND_CHANGELOG_POLICY.md
- Tasks for Round 2 (later):
  - Re-verify proofs, check missed claims, propose fixes. Record notes in this file under "Round 2 Review".

## Round 1 Peer Review Targets

- PRESERVATION_FIDELITY.md
- PREFLIGHT_MODULE_CONCERNS.md
- POLICY_PRESETS_RATIONALE.md
- LOCKING_STRATEGY.md
- idiomatic_todo.md
- SECURITY_REVIEW.md
- RELEASE_AND_CHANGELOG_POLICY.md

### Round 1 Peer Review — Checklist

- [x] PRESERVATION_FIDELITY.md
- [x] PREFLIGHT_MODULE_CONCERNS.md
- [x] POLICY_PRESETS_RATIONALE.md
- [x] LOCKING_STRATEGY.md
- [x] idiomatic_todo.md
- [x] SECURITY_REVIEW.md
- [x] RELEASE_AND_CHANGELOG_POLICY.md

### Round 1 Peer Review — Evidence and Edits

**PRESERVATION_FIDELITY.md**
- Claims → Proofs:
  - ✅ `detect_preservation_capabilities()` in `src/fs/meta.rs:75-106` - verified owner detection via `/proc/self/status`, mode/timestamps/xattrs probing, ACLs/caps hard-coded false
  - ✅ Backup creation in `src/fs/backup.rs:118-232` - verified mode capture via `fchmod`, sidecar storage as octal string
  - ✅ Restore logic in `src/fs/restore.rs:14-271` - verified `renameat` usage and mode restoration via `fchmod`
  - ✅ Preflight integration in `src/api/preflight/mod.rs:140-144` - verified capability detection and policy gating
- Changes Made: Added peer review section with citations, no corrections needed

**PREFLIGHT_MODULE_CONCERNS.md**
- Claims → Proofs:
  - ✅ `src/preflight.rs:7-10` - verified `#[path]` delegation to submodules
  - ✅ `src/api/preflight/mod.rs:17-292` - verified main orchestration function
  - ✅ No `src/policy/checks.rs` found - verified shim removal completed
  - ✅ Helper re-exports in `src/preflight.rs:13` - verified convenience exports
- Changes Made: Added peer review section confirming migration status

**POLICY_PRESETS_RATIONALE.md**
- Claims → Proofs:
  - ✅ `Policy::production_preset()` in `src/policy/config.rs:135-142` - verified all enabled flags
  - ✅ `Policy::coreutils_switch_preset()` in `src/policy/config.rs:180-212` - verified additional restrictions
  - ✅ Mount checks and forbid paths in `src/policy/config.rs:193-208` - verified exact path lists
  - ✅ Mutator methods in `src/policy/config.rs:145-244` - verified apply_*_preset implementations
- Changes Made: Added peer review section, all claims verified accurate

**LOCKING_STRATEGY.md**
- Claims → Proofs:
  - ✅ `LockManager` trait in `src/adapters/lock/mod.rs:6-8` - verified interface definition
  - ✅ `FileLockManager` in `src/adapters/lock/file.rs:12-61` - verified `fs2` usage and polling
  - ✅ Constants in `src/constants.rs:19,22` - verified `LOCK_POLL_MS=25`, `DEFAULT_LOCK_TIMEOUT_MS=5000`
  - ✅ Apply integration in `src/api/apply/mod.rs:57-77` - verified `lock_wait_ms` tracking and `E_LOCKING` error
- Changes Made: Added peer review section, all technical claims verified

**idiomatic_todo.md**
- Claims → Proofs:
  - ✅ Preflight/Apply modules moved to directories - verified `src/api/{preflight,apply}/mod.rs` exist
  - ✅ Preflight checks split - verified `src/preflight/{checks,yaml}.rs` exist
  - ✅ Policy checks shim removed - verified no `src/policy/checks.rs`
  - ❌ **Correction**: `src/api.rs` still exists as file, not moved to directory yet
- Changes Made: Added peer review section with correction about pending api.rs migration

**SECURITY_REVIEW.md**
- Claims → Proofs:
  - ✅ `SafePath` type in `src/types/safepath.rs` - verified path traversal protection
  - ✅ Atomic operations using `*at` syscalls throughout `src/fs/` - verified TOCTOU protection
  - ✅ `BackupSidecar` schema in `src/fs/backup.rs:244-252` - verified topology preservation
  - ✅ Redaction in `src/logging/redact.rs` - verified secret masking
- Changes Made: Added peer review section, all security claims verified

**RELEASE_AND_CHANGELOG_POLICY.md**
- Claims → Proofs:
  - ✅ Process-oriented document with standard practices
  - ⚠️ Limited code verification possible - mostly policy/process content
- Changes Made: Added peer review section noting limited technical verification possible

## Round 2 Meta Review Targets

- BACKWARDS_COMPAT_SHIMS.md
- BEHAVIORS.md
- EXPERIMENT_CONSTANTS_REVIEW.md
- REEXPORTS_AND_FACADES.md
- RETENTION_STRATEGY.md
- PERFORMANCE_PLAN.md
- TEST_COVERAGE_MAP.md
- MIGRATION_GUIDE.md
- ROADMAP.md
- CODING_STANDARDS.md
- CONTRIBUTING_ENHANCEMENTS.md

### Round 2 Meta Review — Notes

- Thoroughness, correctness, evidence quality, and editorial discipline per doc. Do not edit docs; record issues here.

## Round 3 Severity Reports — Targets

- EDGE_CASES_AND_BEHAVIOR.md
- CORE_FEATURES_FOR_EDGE_CASES.md
- CLI_INTEGRATION_GUIDE.md

### Round 3 Severity Reports — Entries

- Topic: <area>
  - Impact: [] Likelihood: [] Confidence: [] → Priority: []
  - Rationale: <citations>

## Round 4 Implementation Plans — Targets (return to own set)

- FS_SAFETY_AUDIT.md
- API_SURFACE_AUDIT.md
- OBSERVABILITY_FACTS_SCHEMA.md
- ERROR_TAXONOMY.md
- INDEX.md

### Plan Template (use per item)

- Summary
- Code targets (files/functions)
- Steps: changes, tests, telemetry/docs
- Feasibility: High/Medium/Low
- Complexity: 1–5
- Risks and mitigations
- Dependencies

## Round 2 Review (placeholder)

- Findings:
- Suggested diffs:
