# Switchyard REORG TODO

This doc is the execution checklist for the module reorganization. It provides:

- A single, pasteable move command (mv-only) to apply the new layout
- A concrete import-fix plan to make the tree compile again
- Remaining follow-ups and guardrails

Run the move command first, then follow the import-fix plan step by step.

## 1) One-shot move command (mv-only)

Run from the repository root. This single command creates target directories and moves files
into their new locations. It uses `mv` by default; if you prefer history-preserving moves, replace
`mv` with `git mv` everywhere.

```bash
bash -lc '
  set -euo pipefail
  cd cargo/switchyard/src
  # Create target directories
  mkdir -p logging fs/meta api/preflight api/apply adapters/lock adapters/ownership policy
  # Moves (renames) — stage-agnostic helpers, adapters, policy, logging
  mv api/audit.rs                 logging/audit.rs
  mv api/fs_meta.rs               fs/meta.rs
  mv api/apply/audit_emit.rs      api/apply/audit_fields.rs
  mv api/apply/gating.rs          policy/gating.rs
  mv api/preflight/report.rs      api/preflight/rows.rs
  mv rescue.rs                    policy/rescue.rs
  mv adapters/lock.rs             adapters/lock/mod.rs
  mv adapters/lock_file.rs        adapters/lock/file.rs
  mv adapters/ownership.rs        adapters/ownership/mod.rs
  mv adapters/ownership_default.rs adapters/ownership/fs.rs
'
```

Notes:

- Intentionally NOT moving `api/preflight.rs` and the root `preflight.rs` in this move-only step to minimize immediate breakage. We'll rename/split them in the import-fix phase.
- If you are in a git repo, consider replacing `mv` with `git mv`.

## 2) Import-fix plan (make it compile)

Apply these edits in order. They are mechanical and can be done with a mix of manual edits and sed.

1) logging module
   - File: `src/logging/mod.rs`
     - Add: `pub mod audit;`
     - Optionally re-export: add `pub use audit::*;` (or specific items) to preserve ergonomics.

2) fs module
   - File: `src/fs/mod.rs`
     - Add: `pub mod meta;`
     - If desired, re-export helpers: `pub use meta::{kind_of, resolve_symlink_target, detect_preservation_capabilities};`

3) adapters module re-exports (to keep public surface stable)
   - File: `src/adapters/mod.rs`
     - Replace existing re-exports with submodule-aware ones:
       - `pub mod lock;` (contains `mod.rs` and `file.rs`)
       - `pub mod ownership;` (contains `mod.rs` and `fs.rs`)
       - Keep: `pub mod attest;`, `pub mod smoke;`, `pub mod path;`
       - Re-exports:
         - `pub use lock::*;`
         - `pub use lock::file::FileLockManager;`
         - `pub use ownership::*;`
         - `pub use ownership::fs::FsOwnershipOracle;`

4) api facade: remove now-stale #[path] indirections
   - File: `src/api.rs`
     - Remove: `#[path = "api/fs_meta.rs"] mod fs_meta;`
     - Remove: `#[path = "api/audit.rs"] mod audit;`
     - Keep the others for now, or convert to plain `mod` once files are co-located.
     - No functional changes to methods.

5) api/apply.rs: adjust imports and module declarations
   - Replace imports:
     - `use super::audit::{...};` → `use crate::logging::audit::{...};`
   - Module includes:
     - Replace `#[path = "apply/gating.rs"] mod gating;` → `use crate::policy::gating;`
     - Replace `#[path = "apply/audit_emit.rs"] mod audit_emit;` → `mod audit_fields;`
   - Keep: `#[path = "apply/handlers.rs"] mod handlers;` (unchanged)

6) api/apply/handlers.rs: import path fixes
   - Replace `use super::super::fs_meta::{...};` → `use crate::fs::meta::{...};`
   - Replace `use super::super::audit::{...};` → `use crate::logging::audit::{...};`
   - Replace `use super::audit_emit::{...};` → `use super::audit_fields::{...};`
   - Replace `use super::super::errors::{...};` → `use crate::api::errors::{...};` (explicit and future-proof)

7) api/preflight.rs (stage orchestrator; not moved yet)
   - Replace `use super::fs_meta::{...};` → `use crate::fs::meta::{...};`
   - Replace `use super::audit::{...};` → `use crate::logging::audit::{...};`
   - Update the internal module include:
     - `#[path = "preflight/report.rs"] mod report;` → `mod rows;`
     - Replace all `report::push_row_emit(...)` → `rows::push_row_emit(...)`
   - Replace `crate::rescue::...` → `crate::policy::rescue::...`

8) policy/gating.rs (moved from api/apply/gating.rs)
   - Update signature to reference the API type path directly:
     - `pub(crate) fn gating_errors<E: FactsEmitter, A: AuditSink>(api: &crate::api::Switchyard<E, A>, plan: &Plan) -> Vec<String>`
   - Keep imports for `FactsEmitter`/`AuditSink` from `crate::logging`.
   - Replace `crate::rescue::...` → `crate::policy::rescue::...`
   - Keep calls to `crate::preflight::ensure_mount_rw_exec` etc. for now (we will split later).

9) api/preflight/report.rs (now at `api/preflight/rows.rs`)
   - No logic change; only ensure the module name is updated in `api/preflight.rs` as per step 7.

10) rescue module
    - All references to `crate::rescue::...` should become `crate::policy::rescue::...`.

11) lib.rs shims (optional but recommended)
    - Add temporary re-export shims to stabilize public API for one minor version:
      - `pub mod preflight { pub use crate::policy::rescue::*; /* once checks split: pub use crate::policy::checks::*; */ pub use crate::api::preflight::rows as yaml; }`
      - Or minimally, add `pub use crate::logging::audit;` and `pub use crate::fs::meta;` if external crates referenced the old paths.

## 3) Bulk search/replace helpers (safe scripts)

From repo root, dry-run with `rg` first, then apply with `sd`/`sed`.

```bash
# Inspect usages that need updates
rg -n "super::super::fs_meta::|super::fs_meta::|crate::api::fs_meta" cargo/switchyard/src
rg -n "super::super::audit::|super::audit::|crate::api::audit" cargo/switchyard/src
rg -n "#[path = \"apply/gating.rs\"]|#[path = \"apply/audit_emit.rs\"]|#[path = \"preflight/report.rs\"]" cargo/switchyard/src
rg -n "crate::rescue::" cargo/switchyard/src

# Apply replacements (BSD/GNU sed may differ; adjust -i accordingly)
# fs_meta → fs::meta
sed -i 's/super::super::fs_meta::/crate::fs::meta::/g' cargo/switchyard/src/api/apply/handlers.rs
sed -i 's/super::fs_meta::/crate::fs::meta::/g' cargo/switchyard/src/api/preflight.rs

# audit module path
sed -i 's/super::super::audit::/crate::logging::audit::/g' cargo/switchyard/src/api/apply/handlers.rs
sed -i 's/super::audit::/crate::logging::audit::/g' cargo/switchyard/src/api/apply.rs cargo/switchyard/src/api/preflight.rs

# module attributes → proper modules
sed -i 's/#\[path = "apply\/gating.rs"\][^\n]*\nmod gating;/use crate::policy::gating;/g' cargo/switchyard/src/api/apply.rs
sed -i 's/#\[path = "apply\/audit_emit.rs"\][^\n]*\nmod audit_emit;/mod audit_fields;/g' cargo/switchyard/src/api/apply.rs
sed -i 's/#\[path = "preflight\/report.rs"\][^\n]*\nmod report;/mod rows;/g' cargo/switchyard/src/api/preflight.rs
sed -i 's/report::push_row_emit/rows::push_row_emit/g' cargo/switchyard/src/api/preflight.rs

# rescue path
sed -i 's/crate::rescue::/crate::policy::rescue::/g' cargo/switchyard/src/api/preflight.rs cargo/switchyard/src/policy/gating.rs
```

Review diffs after each batch.

## 4) Follow-ups (post-compile tasks)

- Split `src/preflight.rs` into:
  - `src/policy/checks.rs`: `ensure_mount_rw_exec`, `check_immutable`, `check_source_trust`
  - `src/api/preflight/yaml.rs`: move `to_yaml()`
- Update references:
  - `policy/gating.rs` to use `crate::policy::checks::*` instead of `crate::preflight::*`
  - Add `pub mod checks;` to `src/policy/mod.rs`
  - In `lib.rs`, consider a compatibility re-export for one minor release:
    - `pub mod preflight { pub use crate::policy::checks::*; pub use crate::api::preflight::yaml::to_yaml; }`
- Remove remaining `#[path = "..."]` attributes from `src/api.rs` as modules become stable.
- Decide whether to re-export `logging::audit` and `fs::meta` in their `mod.rs` for ergonomics.
- Update and/or add unit tests:
  - `policy/gating` unit tests for STOP/warn decisions
  - Smoke tests still pass via existing integration coverage

## 5) Order of operations recap

1) Run the one-shot move command (section 1).
2) Apply import-fix plan (section 2) — suggested order is already topologically sorted.
3) Run `cargo check` in `cargo/switchyard/`.
4) Iterate with bulk search/replace scripts (section 3) until compile succeeds.
5) Execute follow-ups (section 4) to complete the split of `preflight.rs` and add shims.

This keeps changes mechanical and low-risk while converging on the target layout.
