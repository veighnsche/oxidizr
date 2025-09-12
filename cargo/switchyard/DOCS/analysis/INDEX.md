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
