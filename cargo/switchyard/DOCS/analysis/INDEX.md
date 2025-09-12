# Analysis Index: Library Polishing

Generated: 2025-09-12

This index lists analysis documents that exist and additional analyses we can perform to polish the Switchyard library. Check off items as they are authored.

## Existing analyses

- [x] Backwards compatibility and shims inventory
  - File: `BACKWARDS_COMPAT_SHIMS.md`
- [x] Re-exports and facades inventory
  - File: `REEXPORTS_AND_FACADES.md`
- [x] Idiomatic cleanups and refactors plan
  - File: `idiomatic_todo.md`
- [x] Experiment constants review (`oxidizr-arch`)
  - File: `EXPERIMENT_CONSTANTS_REVIEW.md`
- [x] Operational edge cases and behavior
  - File: `EDGE_CASES_AND_BEHAVIOR.md`

- [x] Core features for edge cases
  - File: `CORE_FEATURES_FOR_EDGE_CASES.md`
- [x] Public API surface audit
  - Enumerate what is exposed via `pub` across modules (`fs`, `types`, `logging`, `policy`, `preflight`, `api`).
  - Classify into stable vs. provisional; propose deprecations.
  - Output: `API_SURFACE_AUDIT.md`.
  - File: `API_SURFACE_AUDIT.md`
- [x] Error taxonomy and exit-code mapping
  - Document all error IDs (`ErrorId`) and exit codes, where they are emitted (`apply`, `preflight`, `gating`).
  - Identify gaps, overlapping semantics, and propose a consistent mapping table.
  - Output: `ERROR_TAXONOMY.md`.
  - File: `ERROR_TAXONOMY.md`
- [x] Observability and facts schema review
  - Inventory facts emitted (apply.attempt/result, preflight rows/summary) and their fields.
  - Validate stability, required fields, and versioning strategy.
  - Output: `OBSERVABILITY_FACTS_SCHEMA.md`.
  - File: `OBSERVABILITY_FACTS_SCHEMA.md`
- [x] Filesystem operations safety audit
  - Review use of `open_dir_nofollow`, `*at` syscalls, `renameat`, `fsync` ordering.
  - Identify any remaining TOCTOU windows or missing fsyncs; recommend helper abstractions.
  - Output: `FS_SAFETY_AUDIT.md`.
  - File: `FS_SAFETY_AUDIT.md`

- [x] Preservation capabilities and restore fidelity
  - Compare `detect_preservation_capabilities()` with actual restore behavior (what is preserved now: mode; what is not: uid/gid/mtime/xattrs/acl/caps).
  - Propose a roadmap for optional extended preservation.
  - Output: `PRESERVATION_FIDELITY.md`.
  - File: `PRESERVATION_FIDELITY.md`

- [x] Policy presets coverage and rationale
  - Explain `production_preset` and `coreutils_switch_preset`: what they enable, why, and when to tweak.
  - Validate that presets align with edge cases; propose additions (e.g., checksum bins preservation list via policy).
  - Output: `POLICY_PRESETS_RATIONALE.md`.
  - File: `POLICY_PRESETS_RATIONALE.md`

- [x] Retention and garbage collection strategy
  - Define default retention strategies (per target/tag keep N, max age, total size caps).
  - Propose CLI utilities for pruning.
  - Output: `RETENTION_STRATEGY.md`.
  - File: `RETENTION_STRATEGY.md`

- [x] Concurrency and locking strategy
  - Document intra-process locking options, recommended lock manager implementations, and guidance for cross-process coordination (with package managers).
  - Output: `LOCKING_STRATEGY.md`.
  - File: `LOCKING_STRATEGY.md`

- [x] Performance profiling plan
  - Identify hotspots (hashing, IO syncs, large directory scans) and profiling methodology.
  - Baseline metrics and optimization targets.
  - Output: `PERFORMANCE_PLAN.md`.
  - File: `PERFORMANCE_PLAN.md`

- [x] Test coverage map
  - Map existing tests to features/components; identify gaps (e.g., EXDEV paths, corrupted sidecar, immutable bit scenarios).
  - Output: `TEST_COVERAGE_MAP.md`.
  - File: `TEST_COVERAGE_MAP.md`

- [x] Security review checklist
  - Threat model (symlink attacks, path traversal, time-of-check/time-of-use).
  - Hardening recommendations (umask, directory perms, audit trail integrity).
  - Output: `SECURITY_REVIEW.md`.
  - File: `SECURITY_REVIEW.md`

- [x] CLI integration best practices
  - How to construct per-experiment policies, choose tags, implement retention, handle PM interactions.
  - Output: `CLI_INTEGRATION_GUIDE.md`.
  - File: `CLI_INTEGRATION_GUIDE.md`

- [x] Migration guide for adopters
  - Path changes, removed shims, and how to update imports.
  - Output: `MIGRATION_GUIDE.md`.
  - File: `MIGRATION_GUIDE.md`

- [x] Changelog template and semantic versioning plan
  - Document versioning policy; define how deprecations and removals are handled.
  - Output: `RELEASE_AND_CHANGELOG_POLICY.md`.
  - File: `RELEASE_AND_CHANGELOG_POLICY.md`

- [x] Coding standards and module layout conventions
  - Document idiomatic module structure (directory modules, submodule rules), re-export policy, naming conventions.
  - Output: `CODING_STANDARDS.md`.
  - File: `CODING_STANDARDS.md`

- [x] Contributor guide enhancements
  - Setup, linting, testing, and common pitfalls.
  - Output: `CONTRIBUTING_ENHANCEMENTS.md`.
  - File: `CONTRIBUTING_ENHANCEMENTS.md`

- [x] Roadmap
  - Prioritized features and refactors for the next milestones.
  - Output: `ROADMAP.md`.
  - File: `ROADMAP.md`

## How to use this index

- Pick a proposed analysis, author the document, and mark it as completed in this index.
- Keep analyses concise and actionable with references to relevant files (`path/to/file.rs`, functions, or modules).

## Round 1 Peer Review (AI 1, 2025-09-12 15:14 +02:00)

- Claims verified
  - Existing analyses listed here are present in the repository at the referenced paths.
    - Proof: Files exist under `cargo/switchyard/DOCS/analysis/` including:
      - `FS_SAFETY_AUDIT.md`, `API_SURFACE_AUDIT.md`, `OBSERVABILITY_FACTS_SCHEMA.md`, `ERROR_TAXONOMY.md`, `INDEX.md`.
      - Additional items like `EDGE_CASES_AND_BEHAVIOR.md`, `CORE_FEATURES_FOR_EDGE_CASES.md`, `PRESERVATION_FIDELITY.md`, etc., all present per this index.
  - Scope descriptions align with code locations.
    - Proof: FS safety maps to `src/fs/**` modules (e.g., `src/fs/atomic.rs`, `src/fs/swap.rs`, `src/fs/restore.rs`). API surface aligns with `src/lib.rs`, `src/api.rs`, `src/fs/mod.rs`. Observability maps to `src/logging/{audit,redact}.rs` and `SPEC/audit_event.schema.json`. Error taxonomy maps to `src/api/errors.rs` and `SPEC/error_codes.toml`.

- Key citations
  - `cargo/switchyard/src/lib.rs`, `cargo/switchyard/src/fs/mod.rs`, `cargo/switchyard/src/logging/audit.rs`, `cargo/switchyard/SPEC/audit_event.schema.json`, `cargo/switchyard/SPEC/error_codes.toml`

- Summary of edits
  - Added Round 1 peer-review section to confirm that the index entries correspond to actual files and align with the intended code/spec scope.

Reviewed and updated in Round 1 by AI 1 on 2025-09-12 15:14 +02:00

## Round 2 Gap Analysis (AI 4, 2025-09-12 15:38 CET)

- **Invariant: Analysis index comprehensively covers all critical aspects of the library for consumer understanding and integration.**
  - **Assumption (from doc):** The document assumes that the index provides a complete and actionable list of analysis documents that cover all essential areas of the Switchyard library, aiding CLI consumers and integrators in understanding the library's features, limitations, and best practices (`INDEX.md:5-6`, `INDEX.md:9-111`).
  - **Reality (evidence):** The index lists a broad range of analysis documents covering API surface, filesystem safety, observability, error handling, and more (`INDEX.md:9-111`). However, it does not explicitly address or propose analysis for package manager interoperability beyond a brief mention in `CLI_INTEGRATION_GUIDE.md` (`INDEX.md:83-86`). There is no dedicated analysis for how Switchyard activations persist or interact with package manager upgrades, a critical consumer concern.
  - **Gap:** The absence of a dedicated analysis for package manager interoperability and activation persistence means that CLI consumers lack comprehensive guidance on ensuring Switchyard modifications survive system updates or handle concurrent package manager operations. This violates the expectation of full coverage of integration challenges.
  - **Mitigations:** Add a proposed analysis item to `INDEX.md` titled 'Package Manager Interoperability and Activation Persistence' focusing on strategies for maintaining Switchyard activations post-upgrade, lock ordering with package managers, and post-upgrade verification. Create this document to detail current behavior, gaps, and recommendations.
  - **Impacted users:** CLI integrators and system administrators who rely on Switchyard for system modifications and expect guidance on maintaining state across package manager operations.
  - **Follow-ups:** Flag this as a medium-severity documentation gap for Round 3. Plan to draft the package manager interoperability analysis in Round 4.

- **Invariant: Analysis index is up-to-date and reflects the current state of library analysis efforts.**
  - **Assumption (from doc):** The document assumes that the index accurately reflects the current state of completed and proposed analyses, providing a reliable roadmap for contributors and consumers (`INDEX.md:5-6`, `INDEX.md:113-116`).
  - **Reality (evidence):** The index marks all listed analyses as completed (`INDEX.md:9-111`), and Round 1 peer review confirms the existence of these files (`INDEX.md:118-134`). However, it does not account for the ongoing multi-round analysis process (Rounds 1-4) or indicate which documents have been updated with peer reviews or gap analyses, potentially leading to outdated perceptions of analysis depth.
  - **Gap:** The index does not dynamically reflect the evolving state of analysis documents through multiple review rounds, missing updates on peer reviews or identified gaps. This violates the consumer expectation of a current and transparent overview of library polishing efforts.
  - **Mitigations:** Update `INDEX.md` to include a section or table summarizing the status of each analysis document with respect to review rounds (e.g., 'Round 1 Reviewed', 'Round 2 Gap Analysis Added'). Add a note on how to check individual documents for the latest review status. Consider automating status updates as part of the review process.
  - **Impacted users:** Contributors and CLI integrators who use the index as a starting point for understanding the library's analysis status and contributing to its development.
  - **Follow-ups:** Flag this as a low-severity documentation gap for Round 3. Plan to implement a status tracking mechanism for the index in Round 4.

Gap analysis in Round 2 by AI 4 on 2025-09-12 15:38 CET
