# Migration Guide for Adopters
**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Prepare for upcoming surface cleanups (FS atoms internalization, preflight helper naming). Provide re-export map and deprecation plan.  
**Inputs reviewed:** API Surface Audit; SPEC §3 Public Interfaces; CODE: `src/fs/mod.rs`  
**Affected modules:** `fs/*`, `preflight/*`

## Round 1 Peer Review (AI 3, 2025-09-12 15:14 CEST)

**Verified Claims:**
- Low-level FS atoms will be internalized (`open_dir_nofollow`, `atomic_symlink_swap`, `fsync_parent_dir`, `is_safe_path`).
- Preflight helper naming may be unified.
- The fs module currently re-exports atomic operations which may be internalized in future versions.
- Backup helpers are available and will remain so.
- The deprecation timeline follows standard practices with documentation, deprecation attributes, and eventual removal.

**Citations:**
- `src/fs/mod.rs:L9-L15` - Current public re-exports that may be internalized
- `src/fs/atomic.rs:L22-L33` - `open_dir_nofollow` implementation
- `src/fs/atomic.rs:L43-L96` - `atomic_symlink_swap` implementation
- `src/fs/atomic.rs:L35-L41` - `fsync_parent_dir` implementation
- `src/fs/paths.rs:L7-L19` - `is_safe_path` implementation
- `src/preflight.rs:L7-L10` - Preflight module structure
- `src/preflight.rs:L13-L14` - Preflight re-exports
- `src/fs/mod.rs:L10` - Backup helper re-export

**Summary of Edits:**
- Added verified claims about the current state of FS atoms and preflight helpers.
- Added citations to specific code locations that implement the described functionality.
- Added a Round 1 Peer Review section with verification details.

Reviewed and updated in Round 1 by AI 3 on 2025-09-12 15:14 CEST

## Summary
- Low-level FS atoms will be internalized (`open_dir_nofollow`, `atomic_symlink_swap`, `fsync_parent_dir`, `is_safe_path`). Use high-level helpers instead.
- Preflight helper naming may be unified; prefer `preflight::checks::ensure_mount_rw_exec` facade going forward.

## Deprecations and Re-exports
- FS module
  - Use `fs::replace_file_with_symlink`, `fs::restore_file`, `fs::restore_file_prev`.
  - Backup helpers remain available: `backup_path_with_tag`, `create_snapshot`, `has_backup_artifacts`.

## Timeline
- v0.x: mark Internal in docs; keep re-exports with `#[doc(hidden)]`.
- v0.(x+1): add `#[deprecated]` attributes to internalized re-exports.
- v1.0: remove deprecated re-exports; stable facade only.

## Acceptance Criteria
- Rustdocs updated and deprecation attributes staged according to the above timeline.

## References
- API_SURFACE_AUDIT.md; CODE: `src/fs/mod.rs`

## Round 2 Gap Analysis (AI 2, 2025-09-12 15:29 CEST)

- **Invariant:** Migration guidance prevents consumer breakage during API changes
- **Assumption (from doc):** Deprecation timeline provides sufficient notice for consumers to adapt to API changes
- **Reality (evidence):** Timeline proposes v0.x doc hiding, v0.(x+1) deprecation attributes, v1.0 removal; however, no communication mechanism exists to notify existing consumers of pending changes beyond documentation
- **Gap:** Consumers may not discover deprecation warnings until build time; no proactive notification for API surface changes
- **Mitigations:** Implement deprecation warnings in library initialization; add CHANGELOG.md notifications; consider semantic versioning with minor version bumps for deprecations
- **Impacted users:** Existing library consumers who may encounter sudden API breakage without advance notice
- **Follow-ups:** Add proactive deprecation notification system; document consumer communication strategy

- **Invariant:** High-level helper stability across internalization changes
- **Assumption (from doc):** Consumers can safely migrate from low-level atoms to high-level helpers without functionality loss
- **Reality (evidence):** Document recommends using `fs::replace_file_with_symlink`, `fs::restore_file` instead of internal atoms; these high-level helpers exist at `src/fs/mod.rs:L11` and `src/fs/mod.rs:L12`; however, no compatibility testing validates equivalent functionality
- **Gap:** Consumers migrating to high-level helpers may encounter behavioral differences not covered by current testing
- **Mitigations:** Add migration compatibility tests; document any behavioral differences between low-level and high-level APIs
- **Impacted users:** Library consumers currently using low-level filesystem atoms directly
- **Follow-ups:** Implement compatibility validation tests; document migration behavior differences

- **Invariant:** Clear API boundary documentation guides consumer choices
- **Assumption (from doc):** Documentation clearly distinguishes public stable API from internal implementation details
- **Reality (evidence):** Document mentions `#[doc(hidden)]` for internal re-exports and deprecation attributes; however, no comprehensive API stability documentation exists to guide consumer API selection
- **Gap:** Consumers may inadvertently depend on internal APIs without understanding stability guarantees
- **Mitigations:** Implement comprehensive API stability documentation; add examples of recommended vs deprecated usage patterns
- **Impacted users:** New library adopters who need guidance on which APIs to depend on for long-term stability
- **Follow-ups:** Create API stability guide with usage examples; implement architectural decision records for API boundaries

Gap analysis in Round 2 by AI 2 on 2025-09-12 15:29 CEST

## Round 3 Severity Assessment (AI 1, 2025-09-12 15:44 +02:00)

- Title: Deprecation communication lacks proactive, codified channel
  - Category: Documentation Gap
  - Impact: 3  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Explicit deprecation annotations and CHANGELOG guidance reduce unexpected breakage for consumers migrating off internalized APIs.
  - Evidence: `src/lib.rs:21` re-exports `policy::rescue` without `#[deprecated]`; `src/adapters/mod.rs:6-9` provides `adapters::lock_file` shim; this guide describes timelines but not concrete comms.
  - Next step: Add `#[deprecated(...)]` to shims; add a MIGRATION section per release in CHANGELOG; include concrete examples in this guide.

- Title: No equivalence tests between low-level atoms and high-level helpers
  - Category: Missing Feature
  - Impact: 3  Likelihood: 2  Confidence: 3  → Priority: 2  Severity: S3
  - Disposition: Implement  LHF: No
  - Feasibility: High  Complexity: 3
  - Why update vs why not: Ensures that migrating to `fs::replace_file_with_symlink`/`restore_file` preserves behavior relative to internal atoms.
  - Evidence: High-level helpers exported in `src/fs/mod.rs:14-15` with no dedicated migration compatibility tests.
  - Next step: Add tests that perform the same operation via atoms vs helpers and assert identical state and facts; document any intentional differences here.

Severity assessed in Round 3 by AI 1 on 2025-09-12 15:44 +02:00
