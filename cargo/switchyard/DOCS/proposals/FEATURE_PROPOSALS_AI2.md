# Feature Proposals — AI 2

Generated: 2025-09-12 16:27 +02:00  
Author: AI 2

## Feature 1: Backup and Sidecar Durability Enhancement

- **Problem statement:** Missing fsync operations risk data loss during crashes, undermining core recovery features. S3 Priority 2 from PRESERVATION_FIDELITY.md Round 3 assessment.

- **User story(s):**
  - As a system administrator, I want backup operations to survive system crashes, so that I can reliably restore from backups during recovery scenarios.
  - As an integrator, I want durability guarantees documented and configurable, so that I can tune performance vs. safety for my environment.

- **Design overview:**
  - APIs (new/changed): Add Policy.require_backup_durability flag (default true)
  - Behavior: Replace path-based operations with directory handle + *at syscalls; add fsync_parent_dir() calls
  - Telemetry/facts: Add backup_durable=true|false field to apply facts
  - Policy flags: require_backup_durability (default true)
  - Docs changes: Update PRESERVATION_FIDELITY.md with durability guarantees

- **Scope (files/functions):**
  - `src/fs/backup.rs::create_snapshot()` - replace std::fs operations with *at syscalls
  - `src/fs/backup.rs::write_sidecar()` - add sync_all() and fsync_parent_dir()
  - `src/policy/config.rs` - add require_backup_durability field
  - `src/api/apply/mod.rs` - emit backup_durable fact

- **Tests:**
  - Unit: Extend existing snapshot tests to verify no regressions
  - Integration: Add crash simulation test with SIGKILL to child process
  - Schema validation: Verify backup_durable field in facts schema

- **Feasibility:** High
- **Complexity:** 2
- **Effort:** M

- **Risks and mitigations:**
  - Performance impact from additional fsync operations → Make configurable via policy flag
  - Platform compatibility with *at syscalls → Already used elsewhere in codebase

- **Dependencies:**
  - SPEC Proposal 3: Backup Durability Requirements

- **Rollout plan:**
  - Immediate implementation with policy control
  - Default enabled for safety, can be disabled for performance-critical scenarios

- **Acceptance criteria:**
  - All backup operations use open_dir_nofollow() + *at syscalls
  - fsync_parent_dir() called after backup and sidecar creation
  - Policy flag controls durability enforcement
  - Apply facts include backup_durable status
  - Integration test verifies backup survival across simulated crashes

- **Evidence:**
  - Code: `src/fs/backup.rs::write_sidecar()` lines 262–270, `src/fs/backup.rs::create_snapshot()` lines 137–151
  - Analysis: `PRESERVATION_FIDELITY.md` Round 3 assessment lines 153–160

## Feature 2: Extended Preservation Implementation

- **Problem statement:** Mode-only preservation limits tool applicability in metadata-sensitive environments. S2 Priority 3 from PRESERVATION_FIDELITY.md Round 3 assessment.

- **User story(s):**
  - As a backup administrator, I want complete metadata preservation (owner, timestamps, xattrs), so that restored files maintain system integrity.
  - As a system migrator, I want configurable preservation levels, so that I can balance compatibility with completeness.

- **Design overview:**
  - APIs (new/changed): Add PreservationTier enum and Policy.preservation_tier field
  - Behavior: Capture and restore extended metadata based on tier and capabilities
  - Telemetry/facts: Add preservation_applied object with per-field success flags
  - Policy flags: preservation_tier (Basic/Extended/Full)
  - Docs changes: Document tier behavior and platform limitations

- **Scope (files/functions):**
  - `src/policy/config.rs` - add PreservationTier enum and policy field
  - `src/fs/backup.rs::create_snapshot()` - capture extended metadata
  - `src/fs/restore.rs::restore_file()` - apply extended metadata
  - `src/types/` - extend BackupSidecar schema with optional fields

- **Tests:**
  - Unit: Test each preservation tier with mock files
  - Integration: Verify extended metadata round-trip
  - Capability: Test graceful degradation when capabilities unavailable

- **Feasibility:** Medium
- **Complexity:** 3
- **Effort:** L

- **Risks and mitigations:**
  - Platform compatibility with xattrs/ownership → Graceful degradation with capability detection
  - Sidecar schema versioning → Coordinate with schema v2 work

- **Dependencies:**
  - SPEC Proposal 1: Preservation Tiers and Sidecar v2 Schema
  - Existing detect_preservation_capabilities() function

- **Rollout plan:**
  - Implement with Basic tier as default (current behavior)
  - Extended/Full tiers opt-in via policy configuration

- **Acceptance criteria:**
  - PreservationTier enum with Basic/Extended/Full levels
  - BackupSidecar v2 schema with optional uid, gid, mtime_sec, mtime_nsec, xattrs fields
  - Capability detection integration
  - Graceful degradation when capabilities unavailable
  - preservation_applied facts with per-field success status

- **Evidence:**
  - Code: `src/fs/backup.rs::create_snapshot()` lines 194–205, `src/fs/restore.rs::restore_file()` lines 129–137
  - Analysis: `PRESERVATION_FIDELITY.md` Round 3 assessment lines 144–151

## Feature 3: Immutable Bit Detection Reliability

- **Problem statement:** lsattr dependency fails in minimal environments, allowing operations on immutable files to proceed undetected. S2 Priority 3 from PREFLIGHT_MODULE_CONCERNS.md Round 3 assessment.

- **User story(s):**
  - As a system operator, I want reliable immutable file detection across all environments, so that preflight checks prevent operations on protected files.
  - As a container user, I want immutable detection to work without external dependencies, so that the tool functions in minimal environments.

- **Design overview:**
  - APIs (new/changed): Add Policy.allow_unreliable_immutable_check flag
  - Behavior: Try ioctl first, fallback to lsattr, report detection method
  - Telemetry/facts: Add immutable_detection_method and immutable_check fields
  - Policy flags: allow_unreliable_immutable_check (default false)
  - Docs changes: Document platform support matrix

- **Scope (files/functions):**
  - `src/preflight/checks.rs::check_immutable()` - implement ioctl detection
  - `src/policy/config.rs` - add allow_unreliable_immutable_check flag
  - `Cargo.toml` - add optional nix dependency for ioctl support

- **Tests:**
  - Unit: Test with mock immutable file using chattr +i
  - Fallback: Test behavior when ioctl unavailable
  - Policy: Test override for unreliable detection

- **Feasibility:** Medium
- **Complexity:** 3
- **Effort:** M

- **Risks and mitigations:**
  - Additional dependency complexity → Make nix dependency optional with feature flag
  - Platform-specific ioctl behavior → Comprehensive fallback chain

- **Dependencies:**
  - SPEC Proposal 2: Immutable Bit Detection Requirements

- **Rollout plan:**
  - Add ioctl detection as primary method
  - Maintain lsattr fallback for compatibility
  - Feature flag for nix dependency

- **Acceptance criteria:**
  - ioctl detection using FS_IOC_GETFLAGS
  - Fallback chain: ioctl → lsattr → unknown
  - Policy flag controls unknown detection handling
  - Preflight facts include detection method and reliability
  - Unit tests cover all detection paths

- **Evidence:**
  - Code: `src/preflight/checks.rs::check_immutable()` lines 20–41
  - Analysis: `PREFLIGHT_MODULE_CONCERNS.md` Round 3 assessment lines 153–160

## Feature 4: Public FS Atoms Security Restriction

- **Problem statement:** Public exposure of low-level FS atoms allows SafePath bypass, risking TOCTOU vulnerabilities. S2 Priority 3 from SECURITY_REVIEW.md and idiomatic_todo.md Round 3 assessments.

- **User story(s):**
  - As a security-conscious integrator, I want low-level FS operations restricted, so that I cannot accidentally bypass safety mechanisms.
  - As a library maintainer, I want to guide users toward safe abstractions, so that the API surface reduces footgun potential.

- **Design overview:**
  - APIs (new/changed): Change pub use to pub(crate) use for low-level atoms
  - Behavior: Maintain high-level helpers as public API
  - Telemetry/facts: No changes to facts
  - Policy flags: No new flags
  - Docs changes: Add migration guide and deprecation notices

- **Scope (files/functions):**
  - `src/fs/mod.rs` - restrict re-exports to pub(crate)
  - `src/fs/atomic.rs`, `src/fs/paths.rs` - verify internal usage
  - Documentation - add migration guidance

- **Tests:**
  - Compile-fail: Test external usage of restricted atoms fails
  - Security: Verify SafePath enforcement at API boundary
  - Equivalence: Ensure high-level helpers provide same functionality

- **Feasibility:** Medium
- **Complexity:** 2
- **Effort:** M

- **Risks and mitigations:**
  - Breaking change for existing consumers → Deprecate first with 1-2 release window
  - Migration complexity → Provide clear migration path and examples

- **Dependencies:**
  - SPEC Proposal 5: Deprecation and CHANGELOG Requirements
  - Coordinate with release policy implementation

- **Rollout plan:**
  - Phase 1: Add deprecation warnings with migration guidance
  - Phase 2: Restrict visibility after deprecation window
  - Maintain high-level helpers as stable public API

- **Acceptance criteria:**
  - Low-level FS atoms restricted to pub(crate)
  - Deprecation warnings with clear migration guidance
  - High-level helpers remain public and functional
  - Compile-fail tests prevent external usage
  - Migration guide documents transition path

- **Evidence:**
  - Code: `src/fs/mod.rs` lines 9–15
  - Analysis: `SECURITY_REVIEW.md` Round 3 assessment lines 108–115, `idiomatic_todo.md` lines 184–191

## Feature 5: Production Preset Adapter Configuration

- **Problem statement:** production_preset() enables safety features but doesn't configure required adapters, leading to runtime failures. S2 Priority 3 from POLICY_PRESETS_RATIONALE.md Round 3 assessment.

- **User story(s):**
  - As a new user, I want production_preset() to include setup examples, so that I can configure adapters without trial and error.
  - As an integrator, I want clear guidance on required adapters, so that I avoid runtime E_LOCKING and E_SMOKE errors.

- **Design overview:**
  - APIs (new/changed): No API changes, documentation enhancement only
  - Behavior: No behavior changes
  - Telemetry/facts: No changes
  - Policy flags: No new flags
  - Docs changes: Add comprehensive Rustdoc examples with adapter setup

- **Scope (files/functions):**
  - `src/policy/config.rs::production_preset()` - add Rustdoc examples
  - Documentation - enhance preset usage guidance

- **Tests:**
  - Doc: Ensure Rustdoc examples compile and run
  - Integration: Verify examples work with real adapters

- **Feasibility:** High
- **Complexity:** 1
- **Effort:** S

- **Risks and mitigations:**
  - Documentation drift → Include examples in CI doc tests
  - Adapter API changes → Keep examples minimal and focused

- **Dependencies:**
  - None

- **Rollout plan:**
  - Immediate documentation enhancement
  - Include in next release notes

- **Acceptance criteria:**
  - Rustdoc examples show minimal LockManager setup
  - Examples demonstrate SmokeTestRunner configuration
  - Doc tests verify examples compile
  - Examples cover common integration patterns

- **Evidence:**
  - Code: `src/policy/config.rs::production_preset()` lines 135–141
  - Analysis: `POLICY_PRESETS_RATIONALE.md` Round 3 assessment lines 131–138

## Feature 6: Lock Acquisition Fairness Enhancement

- **Problem statement:** Fixed-interval polling without backoff can cause herding and contention spikes. S3 Priority 2 from LOCKING_STRATEGY.md Round 3 assessment.

- **User story(s):**
  - As a system operator, I want fair lock acquisition under contention, so that processes don't experience indefinite delays.
  - As a performance engineer, I want reduced lock contention overhead, so that high-load scenarios perform predictably.

- **Design overview:**
  - APIs (new/changed): No public API changes
  - Behavior: Add exponential backoff or jitter to polling loop
  - Telemetry/facts: Add lock_attempts field to apply facts
  - Policy flags: No new flags (could add backoff configuration later)
  - Docs changes: Document fairness improvements

- **Scope (files/functions):**
  - `src/adapters/lock/file.rs` - update polling loop with backoff
  - `src/api/apply/mod.rs` - emit lock_attempts telemetry

- **Tests:**
  - Unit: Test backoff behavior with mock time
  - Stress: Multi-process contention test
  - Telemetry: Verify lock_attempts emission

- **Feasibility:** Medium
- **Complexity:** 2
- **Effort:** M

- **Risks and mitigations:**
  - Increased lock acquisition time → Use bounded exponential backoff
  - Complexity in testing → Mock time for deterministic tests

- **Dependencies:**
  - None

- **Rollout plan:**
  - Implement with conservative backoff parameters
  - Monitor performance impact in testing

- **Acceptance criteria:**
  - Exponential backoff or jitter in FileLockManager polling
  - lock_attempts field in apply.attempt facts
  - Stress test shows improved fairness under contention
  - No regression in single-process lock acquisition time

- **Evidence:**
  - Code: `src/adapters/lock/file.rs` lines 50–56
  - Analysis: `LOCKING_STRATEGY.md` Round 3 assessment lines 137–144

---

Proposals authored by AI 2 on 2025-09-12 16:27 +02:00
