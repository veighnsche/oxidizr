# Preflight Module Structure: Concerns, Overlaps, and Recommendations

This document explains the current "triple preflight" layout, whether there is parallel or overlapping functionality, and proposes a cleanup plan.

Relevant paths:

- `cargo/switchyard/src/preflight.rs`
- `cargo/switchyard/src/api/preflight.rs`
- `cargo/switchyard/src/api/preflight/` (currently `rows.rs`)
- `cargo/switchyard/src/policy/checks.rs` (compatibility shim re-exporting `crate::preflight` checks)

## What each module does (today)

- `src/preflight.rs`
  - Contains generic preflight checks and a YAML exporter:
    - `ensure_mount_rw_exec(path: &Path) -> Result<(), String>`
    - `check_immutable(path: &Path) -> Result<(), String>`
    - `check_source_trust(source: &Path, force: bool) -> Result<(), String>`
    - `to_yaml(report: &PreflightReport) -> String`
  - Audience: low-level helpers shared by policy and API; YAML exporter for tests/artifacts.

- `src/api/preflight.rs`
  - Orchestrates the preflight stage for an API `Plan` and emits facts:
    - Runs policy gating (mounts, immutability, trust, ownership, allow/forbid roots).
    - Probes preservation capability (`fs::meta` helpers).
    - Emits one preflight row per action and a summary with error mapping.
    - Stable-order returns `PreflightReport` for `preflight::to_yaml()` export.
  - Depends on `src/api/preflight/rows.rs` for per-row construction and audit emission.

- `src/api/preflight/rows.rs`
  - Internal helper for row construction and audit emission. Not a separate stage.

- `src/policy/checks.rs`
  - Transitional shim:
    - `pub use crate::preflight::{check_immutable, check_source_trust, ensure_mount_rw_exec};`
  - Purpose: allow callers to import checks from `crate::policy::checks` while code migrates away from `crate::preflight`.

## Are there parallel features or overlapping mechanisms?

- **No functional duplication of the preflight stage.** Only `src/api/preflight.rs` implements the preflight stage over a `Plan` and emits facts/rows.
- **There is naming overlap and a layering smell:**
  - `src/preflight.rs` (helpers) vs `src/api/preflight.rs` (stage) both use the name "preflight" which can imply two stages. In reality, one is stage orchestration, the other is a helpers+exporter grab-bag.
  - `src/policy/checks.rs` re-exports helpers from `crate::preflight`, so checks are effectively addressable via two namespaces: `crate::preflight::*` and `crate::policy::checks::*`.
- **Risk of drift, not active duplication:** because the helpers are consumed via multiple import paths, future edits could modify one area while call sites think they are using a differently "owned" module.

## Concrete overlaps

- **Checks exposed twice**
  - Via `crate::preflight::{ensure_mount_rw_exec, check_immutable, check_source_trust}`.
  - Via `crate::policy::checks::{ensure_mount_rw_exec, check_immutable, check_source_trust}` (re-export).
- **Naming confusion**
  - The word "preflight" refers both to:
    - The API preflight stage (`src/api/preflight.rs`), and
    - A file with low-level checks and a YAML exporter (`src/preflight.rs`).

## Why this is a concern

- **Discoverability**: Engineers may search for "preflight" and land in the helper file instead of the API stage, or vice versa.
- **API shape drift**: Re-exports can mask the true ownership of a function, making it unclear where to evolve contracts or add tests.
- **Docs/readability**: External readers may assume duplicated stages or parallel logic.

## Recommendations (clean layering)

1. **Make ownership explicit**
   - Move check functions into a clearly owned module under `policy/` or `preflight/`:
     - Option A (minimal):
       - Keep checks in `src/preflight.rs`, but stop re-exporting them from `policy/checks.rs` once call sites are migrated.
       - Document `src/preflight.rs` as "Preflight checks and YAML exporter".
     - Option B (clean split):
       - Create `src/preflight/mod.rs` with:
         - `src/preflight/checks.rs` (the three functions)
         - `src/preflight/yaml.rs` (`to_yaml()`)
       - Update `src/api/preflight.rs` to import from `crate::preflight::checks`.
       - Delete `policy/checks.rs` re-export after migration.

2. **Rename for clarity**
   - If Option B is not taken, consider renaming `src/preflight.rs` to `src/preflight_checks.rs` and moving `to_yaml()` into `src/preflight_yaml.rs`.
   - Keep `src/api/preflight.rs` as the only module that represents the *stage*.

3. **Add module-level docs**
   - At the top of `src/api/preflight.rs`, add: "This is the preflight stage orchestrator. It consumes low-level checks from `preflight::checks` and emits per-action rows and summary facts."
   - At the top of `src/preflight/*`, add: "These are helper functions and exporters; this is not the stage itself."

4. **Remove the shim**
   - Once all call sites import checks from the canonical path, delete `src/policy/checks.rs` to avoid shadowed imports.

## Non-goals / what to keep as-is

- `api/preflight/rows.rs` is an internal helper to the API stage and should remain private to that stage.
- `preflight::to_yaml()` being separate from the API stage is fine; keep that decoupling to allow tests and fixtures to export reports without pulling API wiring.

## Migration plan (small PRs)

- **PR1**: Introduce `preflight/mod.rs` with `checks.rs` and `yaml.rs`. Move functions from `src/preflight.rs` accordingly. Add module docs to both `api/preflight.rs` and `preflight/mod.rs`.
- **PR2**: Update all imports to `crate::preflight::checks::*`. Keep `policy/checks.rs` during this PR as a re-export.
- **PR3**: Delete `policy/checks.rs` re-export. Run grep to ensure no remaining `crate::policy::checks` imports for these helpers.
- **PR4 (optional)**: If keeping a single-file design, rename files for clarity instead of a module split, and update docs accordingly.

## Quick answers

- **Is there parallel preflight logic?** No. Only `src/api/preflight.rs` runs the preflight stage. The others are helpers.
- **Is there overlap?** Yes in naming and import paths. Checks are accessible via two namespaces due to a re-export shim, which can confuse ownership and evolveability.
- **Is this harmful now?** Not functionally, but it increases cognitive load and the risk of drift. The cleanup above lowers that risk and clarifies responsibility.

## Round 1 Peer Review (AI 2, 2025-09-12 15:01 +02:00)

**Claims Verified:**
- ✅ `src/preflight.rs` exists as a delegator using `#[path]` attributes to `preflight/checks.rs` and `preflight/yaml.rs` (L7-10)
- ✅ `src/api/preflight/mod.rs` orchestrates the preflight stage and emits facts (L17-292), depends on `rows.rs` helper (L15)
- ✅ No `src/policy/checks.rs` file found - the re-export shim has been removed as claimed in the migration plan
- ✅ `src/preflight.rs` re-exports common helpers: `check_immutable`, `check_source_trust`, `ensure_mount_rw_exec` (L13)

**Key Citations:**
- `src/preflight.rs:7-10`: Uses `#[path]` attributes for submodules
- `src/api/preflight/mod.rs:17`: Main preflight orchestration function
- `src/preflight/checks.rs`: Contains the actual check implementations
- File system shows no `src/policy/checks.rs` exists

**Summary of Edits:** Verified that the "triple preflight" concern is accurate but the migration appears partially complete - the policy/checks.rs shim has been removed. The document correctly identifies the naming overlap and layering concerns.

Reviewed and updated in Round 1 by AI 2 on 2025-09-12 15:01 +02:00
