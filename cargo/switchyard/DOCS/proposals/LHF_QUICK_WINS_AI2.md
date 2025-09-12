# Low-Hanging Fruit (Quick Wins) — AI 2

Generated: 2025-09-12 16:27 +02:00  
Author: AI 2

## Quick Win 1: Add Lock Backend Telemetry Field

- **Type:** Telemetry
- **Change:**
  Add `lock_backend` field to apply.attempt and apply.result facts in `src/api/apply/mod.rs`. Field should identify the lock implementation type ("file", "redis", etc.) or "none" when no LockManager is configured. This enables ops teams to correlate lock contention with specific backend implementations.
- **Scope (files):**
  - `src/api/apply/mod.rs` (facts emission around lines 355–357)
- **Why now:**
  S4 Priority 1 finding provides immediate diagnostic value with zero risk and minimal code change.
- **Time estimate:** < 1 day
- **Risk:** Low
- **Acceptance criteria:**
  - lock_backend field present in apply facts
  - Field correctly identifies backend type or "none"
  - Facts schema validation passes
- **Evidence:**
  - Code: `src/api/apply/mod.rs` lines 355–357
  - Analysis: `LOCKING_STRATEGY.md` Round 3 assessment lines 128–135

## Quick Win 2: Add YAML Export Preservation Fields

- **Type:** Telemetry
- **Change:**
  Update `src/preflight/yaml.rs::to_yaml()` to include `preservation` and `preservation_supported` fields in YAML output. These fields are already present in facts but missing from YAML exports, limiting consumer decision-making capability.
- **Scope (files):**
  - `src/preflight/yaml.rs` (to_yaml function around lines 11–25)
- **Why now:**
  S3 Priority 2 finding improves usability with simple field addition and no breaking changes.
- **Time estimate:** a few hours
- **Risk:** Low
- **Acceptance criteria:**
  - preservation and preservation_supported fields in YAML output
  - YAML structure remains backward compatible
  - Unit test verifies field presence
- **Evidence:**
  - Code: `src/preflight/yaml.rs::to_yaml()` lines 11–25
  - Analysis: `PREFLIGHT_MODULE_CONCERNS.md` Round 3 assessment lines 162–169

## Quick Win 3: Add Module-Level Documentation Clarity

- **Type:** Docs
- **Change:**
  Add comprehensive module-level documentation to `src/preflight.rs` and `src/api/preflight/mod.rs` clarifying their distinct roles: preflight.rs provides helper functions while api/preflight/mod.rs orchestrates the preflight stage. Include clear ownership boundaries and usage examples.
- **Scope (files):**
  - `src/preflight.rs` (module-level docs)
  - `src/api/preflight/mod.rs` (module-level docs)
- **Why now:**
  S4 Priority 1 finding reduces contributor confusion with zero code changes and immediate clarity improvement.
- **Time estimate:** a few hours
- **Risk:** Low
- **Acceptance criteria:**
  - Clear module-level docs explaining helper vs orchestrator roles
  - Usage examples for each module
  - Contributor onboarding friction reduced
- **Evidence:**
  - Code: Module structure ambiguity between files
  - Analysis: `PREFLIGHT_MODULE_CONCERNS.md` Round 3 assessment lines 171–178

## Quick Win 4: Add Production Preset Rustdoc Examples

- **Type:** Docs
- **Change:**
  Add comprehensive Rustdoc examples to `src/policy/config.rs::production_preset()` showing minimal LockManager and SmokeTestRunner configuration. Include common integration patterns and error handling to prevent runtime E_LOCKING and E_SMOKE failures.
- **Scope (files):**
  - `src/policy/config.rs` (production_preset function documentation)
- **Why now:**
  S2 Priority 3 finding improves new user onboarding with high-value documentation enhancement.
- **Time estimate:** < 1 day
- **Risk:** Low
- **Acceptance criteria:**
  - Rustdoc examples show minimal adapter setup
  - Examples compile and pass doc tests
  - Common integration patterns documented
- **Evidence:**
  - Code: `src/policy/config.rs::production_preset()` lines 135–141
  - Analysis: `POLICY_PRESETS_RATIONALE.md` Round 3 assessment lines 131–138

## Quick Win 5: Fix allow_unlocked_commit Documentation Mismatch

- **Type:** Docs
- **Change:**
  Align documentation with code reality for `allow_unlocked_commit` default value. Either update the docstring in `src/policy/config.rs` to reflect the actual default (false) or change the code to match the documented default (true). Add unit test to assert the intended behavior.
- **Scope (files):**
  - `src/policy/config.rs` (docstring lines 62–66 and Default impl line 106)
  - Test file for default assertion
- **Why now:**
  S4 Priority 1 finding prevents developer confusion during setup with simple alignment fix.
- **Time estimate:** a few hours
- **Risk:** Low
- **Acceptance criteria:**
  - Documentation matches code behavior
  - Unit test asserts intended default
  - No confusion for developers testing Commit mode
- **Evidence:**
  - Code: `src/policy/config.rs` docstring vs Default impl mismatch
  - Analysis: `LOCKING_STRATEGY.md` Round 3 assessment lines 155–162

## Quick Win 6: Add Deprecation Attributes to Legacy Shims

- **Type:** Refactor
- **Change:**
  Add `#[deprecated]` attributes with migration guidance to legacy shims in `src/adapters/mod.rs` (adapters::lock_file::*) and low-level FS exports in `src/fs/mod.rs`. Include clear migration paths and target removal versions in deprecation messages.
- **Scope (files):**
  - `src/adapters/mod.rs` (shim lines 6–9)
  - `src/fs/mod.rs` (re-exports lines 9–15)
- **Why now:**
  S2 Priority 3 finding prevents integration breakage with simple attribute addition and clear communication.
- **Time estimate:** a few hours
- **Risk:** Low
- **Acceptance criteria:**
  - Deprecation attributes with clear migration guidance
  - Target removal versions specified
  - Compiler warnings guide users to correct APIs
- **Evidence:**
  - Code: `src/adapters/mod.rs` lines 6–9, `src/fs/mod.rs` lines 9–15
  - Analysis: `RELEASE_AND_CHANGELOG_POLICY.md` Round 3 assessment lines 84–91

## Quick Win 7: Add Restore Readiness Telemetry

- **Type:** Telemetry
- **Change:**
  Add `restore_ready=true|false` field to preflight rows based on `has_backup_artifacts()` check in `src/api/preflight/mod.rs`. This helps users understand when restore operations will succeed and guides retention policy decisions.
- **Scope (files):**
  - `src/api/preflight/mod.rs` (preflight row generation)
  - `src/fs/backup.rs` (has_backup_artifacts function integration)
- **Why now:**
  S3 Priority 2 finding improves usability by preventing unexpected restore failures with simple telemetry addition.
- **Time estimate:** < 1 day
- **Risk:** Low
- **Acceptance criteria:**
  - restore_ready field in preflight rows
  - Field accurately reflects backup artifact availability
  - Users can predict restore success before attempting
- **Evidence:**
  - Code: `src/fs/backup.rs` has_backup_artifacts lines 234–242
  - Analysis: `PRESERVATION_FIDELITY.md` Round 3 assessment lines 162–169

## Quick Win 8: Add Environment Sanitization Flag Accuracy

- **Type:** Config
- **Change:**
  Fix optimistic `env_sanitized=true` claim in `src/logging/audit.rs::ensure_provenance()` by either implementing actual PATH/locale sanitization or setting the flag based on real sanitizer results. Add `env_vars_checked=true|false` field for transparency.
- **Scope (files):**
  - `src/logging/audit.rs` (ensure_provenance function lines 210–219)
  - `src/policy/rescue.rs` (potential sanitizer helper)
- **Why now:**
  S4 Priority 1 finding prevents security assessment confusion with simple flag accuracy improvement.
- **Time estimate:** a few hours
- **Risk:** Low
- **Acceptance criteria:**
  - env_sanitized flag reflects actual sanitization status
  - env_vars_checked field provides transparency
  - No misleading security claims in facts
- **Evidence:**
  - Code: `src/logging/audit.rs::ensure_provenance()` lines 210–219
  - Analysis: `SECURITY_REVIEW.md` Round 3 assessment lines 126–133

## Quick Win 9: Add Lock Path Standardization Helper

- **Type:** Config
- **Change:**
  Add `Policy::default_lock_path(root)` helper in `src/policy/config.rs` to standardize lock file paths (e.g., `<root>/.switchyard/lock`). Update documentation and examples to promote per-root locking and prevent path collisions in multi-tenant environments.
- **Scope (files):**
  - `src/policy/config.rs` (new helper function)
  - Documentation updates for lock path conventions
- **Why now:**
  S3 Priority 2 finding prevents lock conflicts with simple helper addition and clear conventions.
- **Time estimate:** < 1 day
- **Risk:** Low
- **Acceptance criteria:**
  - Policy helper provides consistent lock paths
  - Documentation promotes per-root locking
  - Examples use standardized paths
- **Evidence:**
  - Code: `src/adapters/lock/file.rs` free-form path acceptance lines 17–19
  - Analysis: `LOCKING_STRATEGY.md` Round 3 assessment lines 146–153

## Quick Win 10: Add Rescue Profile Detail Enhancement

- **Type:** Telemetry
- **Change:**
  Extend `src/policy/rescue.rs` to expose `rescue_found_count` and optionally `rescue_missing` tool names in preflight summary facts. Update `src/api/preflight/mod.rs` to include these fields while considering redaction policy for tool names.
- **Scope (files):**
  - `src/policy/rescue.rs` (expose count and missing tools)
  - `src/api/preflight/mod.rs` (include in summary facts lines 251–270)
- **Why now:**
  S4 Priority 1 finding improves readiness assessment for reliability teams with low-risk telemetry enhancement.
- **Time estimate:** a few hours
- **Risk:** Low
- **Acceptance criteria:**
  - rescue_found_count in preflight summary
  - Optional rescue_missing tool names (redaction-aware)
  - Better margin assessment for rescue readiness
- **Evidence:**
  - Code: `src/api/preflight/mod.rs` summary lines 251–270
  - Analysis: `POLICY_PRESETS_RATIONALE.md` Round 3 assessment lines 158–165

## Quick Win 11: Create Crate-Level CHANGELOG

- **Type:** Docs
- **Change:**
  Create `cargo/switchyard/CHANGELOG.md` following standard format with sections: Added, Changed, Deprecated, Removed, Fixed, Security. Add CI gate using `cargo public-api` to require changelog updates when public API changes are detected.
- **Scope (files):**
  - `cargo/switchyard/CHANGELOG.md` (new file)
  - CI configuration for public API diff detection
- **Why now:**
  S3 Priority 2 finding improves upgrade transparency with standard changelog practices and automated enforcement.
- **Time estimate:** < 1 day
- **Risk:** Low
- **Acceptance criteria:**
  - CHANGELOG.md exists with proper sections
  - CI enforces changelog updates on API changes
  - Standard format for consumer upgrade decisions
- **Evidence:**
  - Analysis: `RELEASE_AND_CHANGELOG_POLICY.md` Round 3 assessment lines 111–118

## Quick Win 12: Add Coreutils Preset Scoping Warning

- **Type:** Config
- **Change:**
  Add preflight rule in `src/api/preflight/mod.rs` to emit STOP with actionable message when `coreutils_switch_preset()` is used but `allow_roots` is empty. Update preset Rustdoc to emphasize the importance of mutation scoping.
- **Scope (files):**
  - `src/api/preflight/mod.rs` (new preflight check)
  - `src/policy/config.rs` (coreutils_switch_preset documentation)
- **Why now:**
  S3 Priority 2 finding prevents accidental system-wide mutations with simple safety check addition.
- **Time estimate:** a few hours
- **Risk:** Low
- **Acceptance criteria:**
  - Preflight STOP when coreutils preset used without allow_roots
  - Clear actionable message for users
  - Updated documentation emphasizes scoping importance
- **Evidence:**
  - Code: `src/policy/config.rs::coreutils_switch_preset()` lines 180–212
  - Analysis: `POLICY_PRESETS_RATIONALE.md` Round 3 assessment lines 149–156

---

Proposals authored by AI 2 on 2025-09-12 16:27 +02:00
