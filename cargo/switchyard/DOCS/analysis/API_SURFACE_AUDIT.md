# Public API Surface Audit

**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Inventory all publicly exposed items across `fs`, `types`, `logging`, `policy`, `preflight`, `api`, and `adapters`; classify stability and propose cleanup/re-exports.  
**Inputs reviewed:** SPEC §3 (Public Interfaces), SPEC §2 (Requirements), PLAN/10-types-traits.md, PLAN/12-api-module.md, PLAN/40-facts-logging.md, CODE under `src/**`  
**Affected modules:** `src/lib.rs`, `src/api.rs`, `src/api/errors.rs`, `src/types/**`, `src/fs/**`, `src/logging/**`, `src/policy/**`, `src/preflight.rs`, `src/adapters/**`, `src/constants.rs`

## Summary

- Switchyard exposes a clean facade via `src/lib.rs` (`pub mod …; pub use api::*;`) with focused core types (`SafePath`, `Plan`, `ApplyMode`, reports) and the `Switchyard` orchestrator.
- Low-level FS atoms (`open_dir_nofollow`, `atomic_symlink_swap`, `fsync_parent_dir`) are publicly re-exported from `fs/` and should be considered Internal; they are footguns for integrators and duplicate internal invariants.
- Adapters’ trait surfaces (LockManager, OwnershipOracle, SmokeTestRunner, Attestor, PathResolver) are stable; default impls (`FileLockManager`, `FsOwnershipOracle`, `DefaultSmokeRunner`) are provisional.
- Logging sinks (`FactsEmitter`, `AuditSink`, `JsonlSink`) and redaction helpers are public but expected to evolve; mark as Provisional and document backward-compat policy.

## Inventory / Findings

Legend: Stability = Stable | Provisional | Internal (consider private)

- Core facade and orchestrator
  - Item: `Switchyard<E,A>` struct — Module: `api` — Stability: Stable — Ref: `src/api.rs`
  - Item: `plan`, `preflight`, `apply`, `plan_rollback_of` — Module: `api` — Stability: Stable — Ref: `src/api.rs`
  - Item: `errors::ApiError`, `errors::ErrorId`, `errors::exit_code_for*` — Module: `api/errors.rs` — Stability: Stable — Ref: `src/api/errors.rs`

- Types (re-exported at crate root via `pub use types::*;`)
  - `ApplyMode`, `PlanInput`, `Action`, `Plan` — Stable — `src/types/plan.rs`
  - `PreflightReport`, `ApplyReport` — Stable — `src/types/report.rs`
  - `SafePath` — Stable — `src/types/safepath.rs`
  - `types::errors::{ErrorKind, Error, Result}` — Provisional (internal to lib users) — `src/types/errors.rs`
  - `types::ids::{plan_id, action_id}` — Provisional — `src/types/ids.rs`

- Policy
  - `Policy` — Stable — `src/policy/config.rs`
  - Presets: `Policy::production_preset*`, `Policy::coreutils_switch_preset*` — Stable — `src/policy/config.rs`
  - `policy::rescue::{verify_rescue*, RescueStatus}` — Provisional — `src/policy/rescue.rs`

- Adapters (re-exported at crate root via `pub use adapters::*;`)
  - Traits: `LockManager`, `LockGuard`, `OwnershipOracle`, `OwnershipInfo`, `Attestor`, `Signature`, `PathResolver`, `SmokeTestRunner`, `SmokeFailure` — Stable — `src/adapters/**`
  - Default impls: `FileLockManager`, `FsOwnershipOracle`, `DefaultSmokeRunner` — Provisional — `src/adapters/**`
  - Shim: `adapters::lock_file::*` (compat alias) — Internal — `src/adapters/mod.rs`

- Filesystem helpers (re-exported via `pub use` in `fs/mod.rs`)
  - High-level: `replace_file_with_symlink`, `restore_file`, `restore_file_prev` — Stable — `src/fs/swap.rs`, `src/fs/restore.rs`
  - Backup helpers: `backup_path_with_tag`, `create_snapshot`, `has_backup_artifacts` — Provisional — `src/fs/backup.rs`
  - Metadata: `detect_preservation_capabilities`, `kind_of`, `resolve_symlink_target`, `sha256_hex_of` — Provisional — `src/fs/meta.rs`
  - Mount inspection: `ProcStatfsInspector`, `ensure_rw_exec` — Provisional — `src/fs/mount.rs`
  - Low-level atoms: `open_dir_nofollow`, `atomic_symlink_swap`, `fsync_parent_dir` — Internal — `src/fs/atomic.rs`
  - Path check: `is_safe_path` — Internal — `src/fs/paths.rs`

- Preflight module
  - `preflight::checks::{ensure_mount_rw_exec, check_immutable, check_source_trust}` — Provisional — `src/preflight/checks.rs`
  - `preflight::to_yaml(report)` — Stable — `src/preflight/yaml.rs`

- Logging
  - `logging::{FactsEmitter, AuditSink, JsonlSink}` — Provisional — `src/logging/facts.rs`
  - `logging::{redact_event, TS_ZERO, ts_for_mode}` — Provisional — `src/logging/redact.rs`
  - Audit emitters in `logging/audit.rs` are crate-internal (not re-exported) — Internal

- Constants
  - `constants::{DEFAULT_BACKUP_TAG, TMP_SUFFIX, FSYNC_WARN_MS, LOCK_POLL_MS, DEFAULT_LOCK_TIMEOUT_MS, NS_TAG, RESCUE_MUST_HAVE, RESCUE_MIN_COUNT}` — Provisional — `src/constants.rs`

## Recommendations

- Tighten FS surface
  1. Mark `fs::open_dir_nofollow`, `fs::atomic_symlink_swap`, `fs::fsync_parent_dir`, and `fs::paths::is_safe_path` as `pub(crate)` and stop re-exporting them from `fs::mod.rs`. Keep only `replace_file_with_symlink`, `restore_file`, `restore_file_prev`, and metadata/backup helpers public.
  2. Document `replace_file_with_symlink` invariants (preconditions and guarantees) in Rustdoc, pointing to SPEC §2.1 and §2.10.

- Logging facade
  3. Keep `FactsEmitter`/`AuditSink` public but mark as Provisional; add a stability note in `logging/mod.rs` Rustdoc referencing SPEC §5 for schema versioning.

- Preflight naming
  4. Avoid duplicate naming with `fs::mount::ensure_rw_exec` vs `preflight::checks::ensure_mount_rw_exec`. Either:
  - Re-export the `fs::mount` helper into preflight and remove the wrapper, or
  - Rename the preflight wrapper to `ensure_mount_is_rw_exec_preflight`.

- Shims and aliases
  5. Mark `adapters::lock_file::*` as deprecated in Rustdoc with a pointer to `adapters::lock::file::*` and a removal timeline.

## Risks & Trade-offs

- Reducing public FS atoms may break power users. Mitigate with a deprecation window and release notes.
- Stabilizing logging may constrain schema evolution; mitigate via versioned schema (already present) and additive-only changes.

## Spec/Docs deltas

- Add a note to SPEC §3.1/§3.2 stating that low-level FS atoms are intentionally not part of the stable public API; only high-level operations are.
- Update module-level Rustdocs to reflect stability classification.

## Acceptance Criteria

- Public docs reflect which items are Stable/Provisional/Internal.
- Low-level FS atoms are no longer publicly re-exported or are clearly marked Internal.
- Preflight helper naming conflict resolved or documented.

## References

- SPEC: §3 Public Interfaces; §2.1 Atomicity; §2.10 Filesystems & Degraded; §5 Audit Facts
- PLAN: 10-types-traits.md; 12-api-module.md; 40-facts-logging.md; 45-preflight.md; 50-locking-concurrency.md
- CODE: `src/lib.rs`, `src/api.rs`, `src/api/errors.rs`, `src/types/**`, `src/fs/**`, `src/logging/**`, `src/policy/**`, `src/preflight.rs`, `src/adapters/**`

## Round 1 Peer Review (AI 1, 2025-09-12 15:14 +02:00)

- Claims verified
  - Crate facade exposes public modules and re-exports API at root.
    - Proof: `src/lib.rs` lines 11–21 declare `pub mod ...` and `pub use api::*;`.
  - Low-level FS atoms are publicly re-exported but implemented as internal-footgun primitives.
    - Proof: `src/fs/mod.rs` lines 9–15 `pub use atomic::{atomic_symlink_swap, fsync_parent_dir, open_dir_nofollow};` alongside higher-level helpers. Implementations in `src/fs/atomic.rs` are low-level.
  - Adapters trait surfaces exist and are re-exported; default impls present.
    - Proof: `src/adapters/mod.rs` re-exports traits and `FileLockManager`, `FsOwnershipOracle`, and smoke runner; traits defined in `adapters/lock/mod.rs`, `adapters/ownership/*`, `adapters/smoke.rs`, `adapters/path.rs`.
  - Logging sinks and redaction helpers are public; audit emitters remain crate-internal.
    - Proof: `src/logging/mod.rs` re-exports `FactsEmitter`, `AuditSink`, `JsonlSink`, `redact_event`, `TS_ZERO`, `ts_for_mode`. Audit helpers in `src/logging/audit.rs` are `pub(crate)` (module is public but items are crate-internal), matching the intent.
  - Preflight naming duality is present (`fs::mount::ensure_rw_exec` vs `preflight::checks::ensure_mount_rw_exec`).
    - Proof: `src/fs/mount.rs::ensure_rw_exec`, re-exported by `src/fs/mod.rs`; and wrapper `src/preflight/checks.rs::ensure_mount_rw_exec` re-exported via `src/preflight.rs`.

- Key citations
  - `src/lib.rs` (facade and re-exports)
  - `src/fs/mod.rs` (public FS surface)
  - `src/adapters/mod.rs`, `src/adapters/lock/mod.rs`, `src/adapters/smoke.rs`, `src/adapters/path.rs`
  - `src/logging/mod.rs`, `src/logging/audit.rs`
  - `src/preflight.rs`, `src/preflight/checks.rs`, `src/fs/mount.rs`

- Summary of edits
  - Added precise code citations to substantiate API exposure and stability classifications. Confirmed that low-level FS atoms are re-exported today and recommended treating them as Internal in docs. Noted the preflight naming duplication for future cleanup.

Reviewed and updated in Round 1 by AI 1 on 2025-09-12 15:14 +02:00

## Round 2 Gap Analysis (AI 4, 2025-09-12 15:38 CET)

- **Invariant: Public API ensures safe usage by preventing misuse of low-level filesystem operations.**
  - **Assumption (from doc):** The document assumes that low-level filesystem atoms like `open_dir_nofollow`, `atomic_symlink_swap`, and `fsync_parent_dir` should be considered internal and not part of the stable public API to prevent misuse by integrators (`API_SURFACE_AUDIT.md:12`, `API_SURFACE_AUDIT.md:65-66`).
  - **Reality (evidence):** These low-level functions are currently publicly re-exported from `fs/mod.rs` (`src/fs/mod.rs:9-15`), making them accessible to external users despite being classified as 'Internal' in the audit (`API_SURFACE_AUDIT.md:47-48`).
  - **Gap:** Exposing low-level filesystem operations publicly allows CLI consumers to bypass higher-level safe abstractions like `replace_file_with_symlink`, potentially leading to unsafe operations or TOCTOU vulnerabilities. This violates the consumer expectation that the API surface protects against misuse.
  - **Mitigations:** Restrict the visibility of low-level FS atoms by marking them as `pub(crate)` in `fs/mod.rs` and removing their re-export. Provide clear documentation in `lib.rs` and SPEC §3.1/§3.2 that only high-level operations are part of the stable API. Consider a deprecation period with warnings for existing users.
  - **Impacted users:** CLI developers who may inadvertently use these low-level functions, risking unsafe filesystem operations without the safety guarantees of higher-level abstractions.
  - **Follow-ups:** Flag this as a medium-severity usability and safety issue for Round 3. Plan to restrict API surface in Round 4 implementation.

- **Invariant: Public API stability is clearly communicated to ensure reliable integration.**
  - **Assumption (from doc):** The document assumes that stability classifications (Stable, Provisional, Internal) are clearly communicated to integrators, allowing them to rely on stable components and anticipate changes in provisional ones (`API_SURFACE_AUDIT.md:18-19`, `API_SURFACE_AUDIT.md:89-92`).
  - **Reality (evidence):** While the audit classifies items in the document, there is no evidence in the codebase (e.g., `src/lib.rs`, `src/fs/mod.rs`, `src/logging/mod.rs`) that stability levels are documented in Rustdoc or other user-facing materials. For instance, `FactsEmitter` and `AuditSink` are marked Provisional in the audit (`API_SURFACE_AUDIT.md:14`, `API_SURFACE_AUDIT.md:55`), but this is not reflected in the code comments or documentation.
  - **Gap:** The lack of explicit stability documentation in the codebase means CLI consumers may assume all public items are stable by default, leading to integration breakage when provisional items evolve. This violates the expectation of transparent API evolution.
  - **Mitigations:** Add stability annotations (e.g., `#[stability::provisional]` or explicit Rustdoc comments) to public items in the codebase, reflecting the classifications in this audit. Update SPEC §3 to include a stability policy section. Include a note in release documentation about provisional components.
  - **Impacted users:** CLI integrators who build on provisional APIs without awareness of potential changes, risking future compatibility issues.
  - **Follow-ups:** Flag this as a medium-severity documentation gap for Round 3. Plan to implement stability annotations and documentation updates in Round 4.

Gap analysis in Round 2 by AI 4 on 2025-09-12 15:38 CET

## Round 3 Severity Assessment (AI 3, 2025-09-12 15:49+02:00)

- **Title:** Unsafe low-level filesystem functions are publicly exposed
- **Category:** API Design (DX/Usability)
- **Impact:** 4  **Likelihood:** 3  **Confidence:** 5  → **Priority:** 3  **Severity:** S2
- **Disposition:** Implement  **LHF:** Yes
- **Feasibility:** High  **Complexity:** 1
- **Why update vs why not:** Exposing low-level, unsafe primitives is an API design flaw that encourages misuse and can lead to security vulnerabilities (e.g., TOCTOU races) if not used correctly. Making them `pub(crate)` is a simple change that hardens the API surface and guides users to the correct, safe abstractions.
- **Evidence:** `src/fs/mod.rs` publicly re-exports `atomic_symlink_swap`, `fsync_parent_dir`, and `open_dir_nofollow`, as noted in the Round 2 analysis.
- **Next step:** Update the visibility of the low-level functions in `src/fs/mod.rs` to `pub(crate)` in Round 4.

- **Title:** Public API items lack stability documentation
- **Category:** Documentation Gap
- **Impact:** 3  **Likelihood:** 5  **Confidence:** 5  → **Priority:** 2  **Severity:** S3
- **Disposition:** Implement  **LHF:** Yes
- **Feasibility:** High  **Complexity:** 2
- **Why update vs why not:** Without explicit stability markers, consumers may unknowingly build upon provisional APIs, leading to breakage and frustration during upgrades. Documenting stability is a low-effort, high-value change that improves the developer experience and sets clear expectations.
- **Evidence:** The codebase lacks Rustdoc annotations or other markers to indicate the `Stable` vs. `Provisional` status of public API items, as noted in the Round 2 analysis.
- **Next step:** Add Rustdoc comments with stability classifications (`Stable`, `Provisional`, `Internal`) to all public modules and items during Round 4.

Severity assessed in Round 3 by AI 3 on 2025-09-12 15:49+02:00
