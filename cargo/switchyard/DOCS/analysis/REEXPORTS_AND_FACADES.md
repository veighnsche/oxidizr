# Re-exports, Facades, and Migration Leftovers

This document enumerates all `pub use` re-exports in the Switchyard crate, classifies them as ergonomics facades vs. backwards-compatibility shims, and suggests actions.

**Verified Claims:**
- The crate root contains two re-exports: `api::*` and `policy::rescue`.
- The fs module re-exports atomic operations, backup helpers, metadata functions, mount inspection, path utilities, and restore functions.
- The logging module re-exports fact emission and redaction utilities.
- The types module re-exports errors, IDs, plan structures, reports, and SafePath utilities.
- The policy module re-exports the Policy configuration structure.
- The preflight module re-exports checks and YAML export functionality.
- The adapters module contains both facade re-exports and a compatibility shim for lock_file.

**Citations:**
- `src/lib.rs:L20-L21` - crate root re-exports
- `src/fs/mod.rs:L9-L15` - filesystem facade re-exports
- `src/logging/mod.rs:L5-L6` - logging facade re-exports
- `src/types/mod.rs:L7-L11` - types facade re-exports
- `src/policy/mod.rs:L5` - policy facade re-export
- `src/preflight.rs:L13-L14` - preflight facade re-exports
- `src/adapters/mod.rs:L11-L17` - adapters facade re-exports

Generated on: 2025-09-12

## Legend

- Facade: Intentional, stable public API surface aggregating internals for ergonomics.
- Shim: Backwards-compatibility layer preserving an old path; should be deprecated/removed over time.

## Inventory

1) Crate root (`src/lib.rs`)

- Re-exports:
  - `pub use api::*;` — Facade (top-level routing to API surface)
  - `pub use policy::rescue;` — Shim (compatibility re-export of `switchyard::policy::rescue`)

2) Filesystem facade (`src/fs/mod.rs`)

- Re-exports:
  - `pub use atomic::{atomic_symlink_swap, fsync_parent_dir, open_dir_nofollow};` — Facade
  - `pub use backup::{backup_path_with_tag, create_snapshot, has_backup_artifacts};` — Facade
  - `pub use meta::{detect_preservation_capabilities, kind_of, resolve_symlink_target, sha256_hex_of};` — Facade
  - `pub use mount::{ensure_rw_exec, ProcStatfsInspector};` — Facade
  - `pub use paths::is_safe_path;` — Facade
  - `pub use restore::{restore_file, restore_file_prev};` — Facade
  - `pub use swap::replace_file_with_symlink;` — Facade

3) Logging facade (`src/logging/mod.rs`)

- Re-exports:
  - `pub use facts::{AuditSink, FactsEmitter, JsonlSink};` — Facade
  - `pub use redact::{redact_event, ts_for_mode, TS_ZERO};` — Facade

4) Types facade (`src/types/mod.rs`)

- Re-exports:
  - `pub use errors::*;` — Facade
  - `pub use ids::*;` — Facade
  - `pub use plan::*;` — Facade
  - `pub use report::*;` — Facade
  - `pub use safepath::*;` — Facade

5) Policy module (`src/policy/mod.rs`)

- Re-exports:
  - `pub use config::Policy;` — Facade
- Note: `policy::checks` shim removed from module graph as part of refactor; file remains but is unused.

6) Preflight façade (`src/preflight.rs`)

- Re-exports:
  - `pub use checks::{check_immutable, check_source_trust, ensure_mount_rw_exec};` — Facade
  - `pub use yaml::to_yaml;` — Facade
- Intent: `preflight.rs` is the public entrypoint; code lives in `preflight/checks.rs` and `preflight/yaml.rs`.

7) Adapters module (`src/adapters/mod.rs`)

- Re-exports:
  - `pub use attest::*;` — Facade
  - `pub use lock::file::FileLockManager;` — Facade
  - `pub use lock::*;` — Facade
  - `pub use ownership::fs::FsOwnershipOracle;` — Facade
  - `pub use ownership::*;` — Facade
  - `pub use path::*;` — Facade
  - `pub use smoke::*;` — Facade
- Compatibility shim:
  - `pub mod lock_file { pub use super::lock::file::*; }`
  - Purpose: preserve legacy path `switchyard::adapters::lock_file::FileLockManager`

## Recommendations

- Shims to consider deprecating/removing:
  - `lib.rs`: `pub use policy::rescue;` — mark as `#[deprecated]` now; remove on next major.
  - `adapters/mod.rs`: `lock_file` namespace — migrate call sites to `switchyard::adapters::FileLockManager`; remove shim after.
- Facades: keep as-is; they define the intended public API surface.

## Searches used

- Regex: `pub use` across the crate.
- Manual inspection of modules with explicit compatibility comments.

## Notes

- If we want to avoid even façade re-exports for preflight, we can drop the `pub use` from `preflight.rs` and have direct imports from `preflight::checks`/`preflight::yaml`. Current setup keeps a single public entrypoint, which is often preferred for discoverability.

## Round 2 Gap Analysis (AI 2, 2025-09-12 15:23 CEST)

- **Invariant:** Stable public API surface across refactoring
- **Assumption (from doc):** Facade re-exports provide stable public API while allowing internal reorganization
- **Reality (evidence):** Current facades at `src/fs/mod.rs:L9-L15`, `src/types/mod.rs:L7-L11`, and others maintain stable paths; however, no API stability testing validates that refactoring preserves consumer imports
- **Gap:** Internal module restructuring could accidentally break public facades without detection
- **Mitigations:** Add API surface integration tests that import all public paths; implement CI checks for API stability across changes
- **Impacted users:** External library consumers who rely on facade paths for ergonomic imports
- **Follow-ups:** Implement automated API surface validation; document facade stability guarantees

- **Invariant:** Clear distinction between facades and compatibility shims
- **Assumption (from doc):** Ergonomic facades should be permanent while compatibility shims are temporary
- **Reality (evidence):** Document clearly identifies `adapters::lock_file` shim at `src/adapters/mod.rs:L88` and `policy::rescue` re-export at `src/lib.rs` as temporary compatibility layers; however, no systematic approach prevents facades from accumulating shim-like behavior over time
- **Gap:** Facade vs shim boundaries may blur without governance; future API evolution could create inconsistent deprecation policies
- **Mitigations:** Establish explicit lifecycle policies for facades vs shims; add documentation tags to distinguish permanent API from compatibility layers
- **Impacted users:** API consumers who need to understand long-term stability of different import paths
- **Follow-ups:** Document facade lifecycle policy; implement architectural decision records for API surface changes

- **Invariant:** Re-export granularity matches consumer usage patterns
- **Assumption (from doc):** Current re-export granularity (e.g., `types::*` vs specific items) reflects actual consumer needs
- **Reality (evidence):** Mixed granularity with `types/mod.rs` using glob re-exports (`pub use errors::*;`) while `fs/mod.rs` uses selective re-exports; no usage analysis validates that granularity matches consumer import patterns
- **Gap:** Over-broad or under-specific re-exports may not align with actual consumer needs; glob exports can lead to namespace pollution
- **Mitigations:** Analyze consumer usage patterns to optimize re-export granularity; consider selective re-exports to avoid namespace pollution
- **Impacted users:** Library consumers who may encounter naming conflicts or missing convenience imports
- **Follow-ups:** Conduct usage analysis of public API imports; refine re-export granularity based on findings

Gap analysis in Round 2 by AI 2 on 2025-09-12 15:23 CEST

## Round 3 Severity Assessment (AI 1, 2025-09-12 15:44 +02:00)

- Title: No API surface stability tests for facade imports
  - Category: Missing Feature
  - Impact: 3  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Prevents accidental breakage during refactors by ensuring public `pub use` imports remain valid.
  - Evidence: Facade re-exports at `src/fs/mod.rs:9-15`, `src/types/mod.rs:57-63`, `src/logging/mod.rs:51-53` with no integration test coverage.
  - Next step: Add a `tests/public_api.rs` that imports all documented public paths; wire into CI.

- Title: Lack of lifecycle policy distinguishing facades vs compatibility shims
  - Category: Documentation Gap
  - Impact: 2  Likelihood: 3  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 1
  - Why update vs why not: Clarifies long-term stability guarantees and deprecation expectations for consumers.
  - Evidence: Shim at `src/adapters/mod.rs:6-9` and crate-root `policy::rescue` re-export at `src/lib.rs:21` without documented lifecycle.
  - Next step: Add an API lifecycle section to SPEC §3 and this doc; tag shims with `#[deprecated]` notes and timelines.

- Title: Over-broad glob re-exports risk namespace pollution
  - Category: DX/Usability
  - Impact: 2  Likelihood: 2  Confidence: 3  → Priority: 2  Severity: S3
  - Disposition: Backlog  LHF: No
  - Feasibility: Medium  Complexity: 3
  - Why update vs why not: Selective re-exports improve clarity but require usage analysis and potential breaking changes.
  - Evidence: `src/types/mod.rs` uses `pub use errors::*;`, `ids::*`, etc.; mixed granularity across modules.
  - Next step: Analyze consumer usage and consider narrowing re-exports in a major/minor with deprecations.

Severity assessed in Round 3 by AI 1 on 2025-09-12 15:44 +02:00
