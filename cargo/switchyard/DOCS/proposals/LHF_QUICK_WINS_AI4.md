# Low-Hanging Fruit (Quick Wins) — AI 4
Generated: 2025-09-12 16:34 CET
Author: AI 4

## Quick Win 1: Improve CLI Integration Guide Examples and Warnings
- Type: Docs
- Change:
  - Update `CLI_INTEGRATION_GUIDE.md` to include detailed `SafePath` usage examples, error handling for invalid paths, and remove references to non-existent functions like `prune_backups` (or mark as 'future work'). Add interim best practices for secure path handling until full `SafePath` enforcement.
- Scope (files):
  - `DOCS/analysis/CLI_INTEGRATION_GUIDE.md`
- Why now:
  - Fast payoff for developer experience by aligning documentation with current API reality; addresses S2 Priority 3 finding from Round 3 assessment, clarifying UX for CLI integrators.
- Time estimate: A few hours
- Risk: Low
- Acceptance criteria:
  - Guide includes `SafePath` examples and error handling.
  - Non-existent functions are removed or marked as 'future work'.
  - Interim best practices for raw `Path` inputs are documented.
- Evidence:
  - Analysis: DOCS/analysis/CLI_INTEGRATION_GUIDE.md Round 3 assessment.
  - Code: `src/types/safepath.rs` (SafePath definition).

## Quick Win 2: Add lock_backend Telemetry to Apply Facts
- Type: Telemetry
- Change:
  - Add `lock_backend` field (e.g., "file", "none") to `apply.attempt` and `apply.result` facts in `src/api/apply/mod.rs` to improve diagnostics for lock contention issues.
- Scope (files):
  - `src/api/apply/mod.rs` (facts emission, lines 355–357)
- Why now:
  - Quick observability win addressing S4 Priority 1 finding from LOCKING_STRATEGY.md Round 3 assessment; enhances fleet-wide analysis for ops teams.
- Time estimate: A few hours
- Risk: Low
- Acceptance criteria:
  - `lock_backend` field present in apply facts.
  - Field identifies backend type or "none" if no LockManager configured.
  - Facts schema validation includes new field.
- Evidence:
  - Analysis: DOCS/analysis/LOCKING_STRATEGY.md Round 3 assessment (lines 128–135).
  - Code: `src/api/apply/mod.rs` (lines 355–357).

## Quick Win 3: Add Module-Level Documentation for Preflight Roles
- Type: Docs
- Change:
  - Add clear module-level documentation to `src/preflight.rs` (helpers) and `src/api/preflight/mod.rs` (stage orchestrator) to disambiguate their roles and reduce contributor confusion.
- Scope (files):
  - `src/preflight.rs`, `src/api/preflight/mod.rs`
- Why now:
  - Addresses S4 Priority 1 finding from PREFLIGHT_MODULE_CONCERNS.md Round 3 assessment; quick clarity win for DX with minimal effort.
- Time estimate: A few hours
- Risk: Low
- Acceptance criteria:
  - Both modules have single-paragraph summaries explaining their responsibilities (helpers vs. stage).
  - Documentation matches current codebase structure.
- Evidence:
  - Analysis: DOCS/analysis/PREFLIGHT_MODULE_CONCERNS.md Round 3 assessment (lines 166–173).
  - Code: `src/preflight.rs`, `src/api/preflight/mod.rs`.

## Quick Win 4: Add YAML Export for Preservation Fields
- Type: Telemetry
- Change:
  - Update `src/preflight/yaml.rs::to_yaml()` to include `preservation` and `preservation_supported` fields in YAML output for better decision-making by consumers.
- Scope (files):
  - `src/preflight/yaml.rs` (lines 11–25)
- Why now:
  - Quick usability fix addressing S3 Priority 2 finding from PREFLIGHT_MODULE_CONCERNS.md Round 3 assessment; enhances preflight report utility.
- Time estimate: A few hours
- Risk: Low
- Acceptance criteria:
  - YAML export includes `preservation` and `preservation_supported` fields.
  - Tests verify correct field inclusion in YAML output.
- Evidence:
  - Analysis: DOCS/analysis/PREFLIGHT_MODULE_CONCERNS.md Round 3 assessment (lines 159–166).
  - Code: `src/preflight/yaml.rs::to_yaml()` (lines 11–25).

## Quick Win 5: Add Production Preset Adapter Setup Examples
- Type: Docs
- Change:
  - Add Rustdoc examples to `production_preset()` in `src/policy/config.rs` demonstrating minimal adapter configuration for `LockManager` and `SmokeTestRunner` to ease onboarding.
- Scope (files):
  - `src/policy/config.rs` (lines 135–141)
- Why now:
  - Quick DX improvement addressing S2 Priority 3 finding from POLICY_PRESETS_RATIONALE.md Round 3 assessment; reduces setup errors for new users.
- Time estimate: A few hours
- Risk: Low
- Acceptance criteria:
  - Rustdoc for `production_preset()` includes adapter setup snippets.
  - Examples are minimal and compile without errors.
- Evidence:
  - Analysis: DOCS/analysis/POLICY_PRESETS_RATIONALE.md Round 3 assessment (lines 83–90).
  - Code: `src/policy/config.rs::production_preset()` (lines 135–141).

## Quick Win 6: Add Deprecation Warnings to Legacy Shims
- Type: Refactor
- Change:
  - Add `#[deprecated]` attributes to legacy shims like `adapters::lock_file::*` in `src/adapters/mod.rs` with migration guidance and target removal version.
- Scope (files):
  - `src/adapters/mod.rs` (lines 6–9)
- Why now:
  - Quick win to signal API transitions, addressing S2 Priority 3 finding from RELEASE_AND_CHANGELOG_POLICY.md Round 3 assessment; prevents future integration issues.
- Time estimate: A few hours
- Risk: Low
- Acceptance criteria:
  - `#[deprecated]` attributes added to legacy shims with clear notes.
  - Compile warnings appear for shim usage.
  - Changelog entry drafted for deprecation notice.
- Evidence:
  - Analysis: DOCS/analysis/RELEASE_AND_CHANGELOG_POLICY.md Round 3 assessment (lines 84–91).
  - Code: `src/adapters/mod.rs` (lines 6–9).

---

Quick Wins authored by AI 4 on 2025-09-12 16:34 CET
