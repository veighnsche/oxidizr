# Policy Presets Coverage & Rationale

**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Explain `production_preset` and `coreutils_switch_preset`: what they enable, why, suggested overrides, and alignment with operational edge-cases.  
**Inputs reviewed:** SPEC §2 (Requirements), PLAN/15-policy-and-adapters.md, CODE: `src/policy/config.rs`, `src/policy/gating.rs`, `src/policy/rescue.rs`, `src/preflight/checks.rs`, analysis `EDGE_CASES_AND_BEHAVIOR.md`  
**Affected modules:** `policy/config.rs`, `policy/gating.rs`, `policy/rescue.rs`, `preflight/checks.rs`, `api/apply/mod.rs`

## Summary

- `Policy::production_preset()` turns on the three pillars for safe commits: rescue verification, process-level locking, and post-apply smoke tests. This matches SPEC’s conservatism and recovery-first mandates.
- `Policy::coreutils_switch_preset()` builds on production defaults and tightens gates for critical paths: no EXDEV degraded fallback, strict ownership, preservation required, explicit allow/forbid path sets.
- Operators should scope the mutation surface by populating `allow_roots` precisely and optionally adding extra mount checks. `forbid_paths` prevents accidents on virtual/volatile filesystems.

## Inventory / Findings

- `Policy` fields (`src/policy/config.rs`):
  - Rescue: `require_rescue` (+ `rescue_exec_check`, `rescue_min_count`) — verified by `policy::rescue::verify_rescue_tools_with_exec_min()`.
  - Locking: `require_lock_manager`, `allow_unlocked_commit` — enforced in `api/apply/mod.rs` with `E_LOCKING` mapping and `lock_wait_ms` emission.
  - Health: `require_smoke_in_commit`, `disable_auto_rollback` — smoke is enforced in Commit, failure → auto-rollback unless disabled.
  - FS degraded mode: `allow_degraded_fs` — controls EXDEV fallback acceptance and corresponding telemetry.
  - Preservation: `require_preservation` — integrates with `detect_preservation_capabilities()` during preflight.
  - Source trust, ownership, path scoping: `force_untrusted_source`, `strict_ownership`, `allow_roots`, `forbid_paths`, `extra_mount_checks`.
  - Restore: `force_restore_best_effort`, `capture_restore_snapshot`.

- Presets
  - `production_preset()` enables: `require_rescue=true`, `rescue_exec_check=true`, `require_lock_manager=true`, `require_smoke_in_commit=true`. All other flags retain safe defaults (e.g., `allow_unlocked_commit=false`).
  - `coreutils_switch_preset()` additionally sets:
    - `allow_degraded_fs=false` (fail on EXDEV rather than degraded link replacement)
    - `strict_ownership=true`
    - `require_preservation=true`
    - `override_preflight=false`, `force_untrusted_source=false`, `force_restore_best_effort=false`
    - `backup_tag="coreutils"`
    - `extra_mount_checks=[/usr, /bin, /sbin, /usr/bin, /usr/sbin]`
    - `forbid_paths=[/proc, /sys, /dev, /run, /tmp]`

## Recommendations

- Baseline guidance
  1. Always start from `production_preset()` for Commit-mode production runs.
  2. For critical system switches (e.g., shell/coreutils), use `coreutils_switch_preset()` and explicitly narrow `allow_roots` to the intended subtree (e.g., `<root>/usr/bin`).

- Environment-specific overrides
  3. Set `allow_degraded_fs=true` only when cross-filesystem topology is unavoidable and you accept non-atomic fallback (facts will include `degraded=true`).
  4. If your fleet runs as non-root and ownership checks would always fail, set `strict_ownership=false` but keep `force_untrusted_source=false` to avoid world-writable sources.
  5. For high-availability scenarios, consider `disable_auto_rollback=true` only with out-of-band safety nets; otherwise keep default auto-rollback.

- Documentation and DX
  6. Add Rustdoc examples for both presets showing minimal configuration snippets and typical `allow_roots` population.
  7. Provide a helper `Policy::apply_coreutils_switch_preset(&mut self)` (already present) and mirror for production: `apply_production_preset` (already present) for mutable updates.

## Risks & Trade-offs

- Overly strict gating may produce false negatives in heterogeneous environments (e.g., missing BusyBox but sufficient GNU tools). Mitigate by tuning `rescue_min_count` and `extra_mount_checks`.
- Disallowing degraded EXDEV may block legitimate cross-filesystem deployments; use per-target policy exceptions if necessary.

## Spec/Docs deltas

- SPEC §2.6 Rescue: clarify the minimum GNU subset and executability checks; include the preset defaults for `rescue_min_count`.
- SPEC §2.10 EXDEV: include operator guidance on when degraded fallback is acceptable and corresponding telemetry expectations.

## Acceptance Criteria

- Preset Rustdocs list all toggled fields and rationale.
- Example snippets compile and demonstrate correct scoping via `allow_roots`.
- Edge-case doc (`EDGE_CASES_AND_BEHAVIOR.md`) cross-references these presets for scenarios like immutable bit or read-only mounts.

## References

- SPEC: §2 Safety/Rescue/Degraded; §3 Public Interfaces
- PLAN: 15-policy-and-adapters.md; 45-preflight.md
- CODE: `src/policy/config.rs`, `src/policy/gating.rs`, `src/policy/rescue.rs`, `src/preflight/checks.rs`, `src/api/apply/mod.rs`

## Round 1 Peer Review (AI 2, 2025-09-12 15:01 +02:00)

**Claims Verified:**
- `Policy::production_preset()` in `src/policy/config.rs:135-142` enables `require_rescue=true`, `rescue_exec_check=true`, `require_lock_manager=true`, `require_smoke_in_commit=true`
- `Policy::coreutils_switch_preset()` in `src/policy/config.rs:180-212` builds on production preset and adds: `allow_degraded_fs=false`, `strict_ownership=true`, `require_preservation=true`, `backup_tag="coreutils"`
- Extra mount checks include `/usr`, `/bin`, `/sbin`, `/usr/bin`, `/usr/sbin` (L193-199)
- Forbid paths include `/proc`, `/sys`, `/dev`, `/run`, `/tmp` (L202-208)
- Both presets have corresponding `apply_*_preset()` mutator methods (L145-151, L215-244)

**Key Citations:**
- `src/policy/config.rs:135-142`: Production preset implementation
- `src/policy/config.rs:180-212`: Coreutils switch preset implementation  
- `src/policy/config.rs:193-199`: Extra mount checks configuration
- `src/policy/config.rs:202-208`: Forbidden paths configuration

**Summary of Edits:** All claims about preset configurations are accurately verified against the codebase. The document correctly describes the policy flags enabled by each preset and their rationale.

Reviewed and updated in Round 1 by AI 2 on 2025-09-12 15:01 +02:00

## Round 2 Gap Analysis (AI 1, 2025-09-12 15:22 +02:00)

- Invariant: Production hardening when using `production_preset()`
  - Assumption (from doc): Enabling the preset sufficiently enforces rescue, locking with bounded wait, and smoke in Commit.
  - Reality (evidence): `Policy::production_preset()` sets `require_rescue=true`, `rescue_exec_check=true`, `require_lock_manager=true`, `require_smoke_in_commit=true` (`src/policy/config.rs` lines 135–141). `apply/mod.rs` enforces E_LOCKING on missing lock (lines 101–131) and E_SMOKE on missing/failing smoke (lines 314–346). Preflight summary records rescue profile status (lines 251–270 in `api/preflight/mod.rs`).
  - Gap: None for preset behavior itself; however, consumers must still configure adapters (LockManager, SmokeTestRunner). Missing adapters produce WARN/fail paths but this dependency may be implicit to users.
  - Mitigations: Document minimal adapter configuration snippets under the preset section; add compile-time examples in Rustdoc.
  - Impacted users: New integrators assuming presets auto-wire adapters.
  - Follow-ups: Add a code example block in `policy/config.rs` docs demonstrating adapter setup.

- Invariant: Dev-friendly default allows unlocked Commit
  - Assumption (from doc comment): `allow_unlocked_commit` defaults to true for development ergonomics (`src/policy/config.rs` lines 62–66 docstring).
  - Reality (evidence): `impl Default for Policy` sets `allow_unlocked_commit = false` (line 106). This mismatch can surprise consumers relying on the documented default.
  - Gap: Spec/doc vs implementation divergence for default behavior.
  - Mitigations: Align either the code or the docs. Prefer setting default to true for dev ergonomics, while ensuring `production_preset()` overrides to hardened settings; or update docs to state default=false and recommend explicit enabling in dev.
  - Impacted users: Developers running quick Commit-mode trials without a LockManager.
  - Follow-ups: Open an issue to reconcile default; add a test asserting intended default; update Rustdoc accordingly.

- Invariant: Coreutils preset fully scopes mutations
  - Assumption (from doc): Using `coreutils_switch_preset()` plus `allow_roots` ensures changes are confined.
  - Reality (evidence): `coreutils_switch_preset()` sets forbids and extra mount checks (lines 193–208) but does not auto-populate `allow_roots`; callers must set it. Gating checks enforced in `src/policy/gating.rs` for `allow_roots`/`forbid_paths` (lines 71–90, 112–131).
  - Gap: Consumers may omit `allow_roots` and unintentionally operate broadly if other gates pass.
  - Mitigations: In `coreutils_switch_preset()`, consider refusing Commit unless `allow_roots` is non-empty (policy flag or runtime check); alternatively, emit a preflight STOP `target outside allowed roots` when `allow_roots` is empty by design.
  - Impacted users: Operators switching critical toolchains who forget to restrict scope.
  - Follow-ups: Add a preflight rule: when preset detected and `allow_roots.is_empty()`, add STOP with actionable message; document in preset Rustdoc.

- Invariant: Rescue profile adequacy is transparent to consumers
  - Assumption (from doc): Rescue availability is binary (available/none).
  - Reality (evidence): Preflight summary includes `rescue_profile: Some("available"|"none")` (lines 251–270) but does not expose the count of found tools or names.
  - Gap: Consumers cannot assess margin (e.g., exactly how many required tools are present) which is useful for readiness and drift detection.
  - Mitigations: Emit `rescue_found_count` and optionally `rescue_missing: [..]` in preflight summary (additive fields) when feasible; keep redaction policy in mind.
  - Impacted users: Site-reliability teams tracking rescue readiness over time.
  - Follow-ups: Extend `policy::rescue` to surface counts and names; add to facts summary.

Gap analysis in Round 2 by AI 1 on 2025-09-12 15:22 +02:00

## Round 3 Severity Assessment (AI 4, 2025-09-12 15:52 CET)

- **Title:** Lack of Adapter Configuration Guidance for Production Preset
  - **Category:** Documentation Gap (Usability)
  - **Impact:** 3  **Likelihood:** 4  **Confidence:** 5  → **Priority:** 3  **Severity:** S2
  - **Disposition:** Implement  **LHF:** Yes
  - **Feasibility:** High  **Complexity:** 1
  - **Why update vs why not:** The `production_preset()` enables critical safety features but doesn't auto-configure required adapters (LockManager, SmokeTestRunner), which can lead to unexpected warnings or failures for new users. Adding documentation with minimal adapter setup examples is a simple, high-value fix to improve onboarding. The cost of inaction is user frustration and potential misconfiguration.
  - **Evidence:** `src/policy/config.rs::production_preset()` sets safety flags but doesn't configure adapters (lines 135–141). `src/api/apply/mod.rs` enforces `E_LOCKING` and `E_SMOKE` on missing adapters (lines 101–131, 314–346).
  - **Next step:** Add Rustdoc examples in `src/policy/config.rs` for `production_preset()` showing minimal adapter configuration snippets. Implement in Round 4.

- **Title:** Mismatch in Default for `allow_unlocked_commit`
  - **Category:** Documentation Gap (DX/Usability)
  - **Impact:** 2  **Likelihood:** 3  **Confidence:** 5  → **Priority:** 1  **Severity:** S4
  - **Disposition:** Implement  **LHF:** Yes
  - **Feasibility:** High  **Complexity:** 1
  - **Why update vs why not:** The documented default for `allow_unlocked_commit` (true for dev ergonomics) contradicts the code (false), potentially confusing developers who rely on docs for quick trials. Aligning docs or code is a low-effort fix that prevents minor usability issues. The cost of inaction is slight confusion during development setup.
  - **Evidence:** `src/policy/config.rs` docstring claims `allow_unlocked_commit` defaults to true (lines 62–66), but `impl Default for Policy` sets it to false (line 106).
  - **Next step:** Update either the code to set `allow_unlocked_commit = true` by default in `src/policy/config.rs` or revise the docstring to reflect the current default. Add a test to assert the intended default. Implement in Round 4.

- **Title:** Incomplete Mutation Scoping in Coreutils Preset Without `allow_roots`
  - **Category:** Missing Feature (Usability)
  - **Impact:** 3  **Likelihood:** 3  **Confidence:** 5  → **Priority:** 2  **Severity:** S3
  - **Disposition:** Implement  **LHF:** No
  - **Feasibility:** Medium  **Complexity:** 2
  - **Why update vs why not:** The `coreutils_switch_preset()` doesn't enforce or auto-populate `allow_roots`, risking broader mutations than intended if users forget to configure it. Adding a safeguard (e.g., requiring non-empty `allow_roots`) prevents accidental overreach, enhancing safety for critical operations. The cost of inaction is potential unintended system changes.
  - **Evidence:** `src/policy/config.rs::coreutils_switch_preset()` sets forbids and mount checks but leaves `allow_roots` empty (lines 180–212). Gating in `src/policy/gating.rs` checks `allow_roots` if set (lines 71–90).
  - **Next step:** Add a preflight rule in `src/api/preflight/mod.rs` to emit a STOP with an actionable message if `coreutils_switch_preset()` is used and `allow_roots` is empty. Update preset Rustdoc to emphasize scoping. Plan for Round 4.

- **Title:** Lack of Transparency in Rescue Profile Details
  - **Category:** Observability (DX/Usability)
  - **Impact:** 2  **Likelihood:** 3  **Confidence:** 5  → **Priority:** 1  **Severity:** S4
  - **Disposition:** Implement  **LHF:** Yes
  - **Feasibility:** High  **Complexity:** 2
  - **Why update vs why not:** Current rescue profile reporting only shows binary availability (`available`/`none`), missing details on tool count or specific missing tools, which limits readiness assessment. Adding count and optional missing tool info is a low-risk enhancement for monitoring. The cost of inaction is minor inconvenience for reliability teams tracking rescue readiness.
  - **Evidence:** `src/api/preflight/mod.rs` summary includes only `rescue_profile: Some("available"|"none")` (lines 251–270), lacking detailed counts or names.
  - **Next step:** Extend `src/policy/rescue.rs` to expose `rescue_found_count` and optionally `rescue_missing` in preflight summary facts in `src/api/preflight/mod.rs`. Ensure redaction policy is considered. Plan for Round 4.

Severity assessed in Round 3 by AI 4 on 2025-09-12 15:52 CET
