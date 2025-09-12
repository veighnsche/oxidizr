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
- ✅ `Policy::production_preset()` in `src/policy/config.rs:135-142` enables `require_rescue=true`, `rescue_exec_check=true`, `require_lock_manager=true`, `require_smoke_in_commit=true`
- ✅ `Policy::coreutils_switch_preset()` in `src/policy/config.rs:180-212` builds on production preset and adds: `allow_degraded_fs=false`, `strict_ownership=true`, `require_preservation=true`, `backup_tag="coreutils"`
- ✅ Extra mount checks include `/usr`, `/bin`, `/sbin`, `/usr/bin`, `/usr/sbin` (L193-199)
- ✅ Forbid paths include `/proc`, `/sys`, `/dev`, `/run`, `/tmp` (L202-208)
- ✅ Both presets have corresponding `apply_*_preset()` mutator methods (L145-151, L215-244)

**Key Citations:**
- `src/policy/config.rs:135-142`: Production preset implementation
- `src/policy/config.rs:180-212`: Coreutils switch preset implementation  
- `src/policy/config.rs:193-199`: Extra mount checks configuration
- `src/policy/config.rs:202-208`: Forbidden paths configuration

**Summary of Edits:** All claims about preset configurations are accurately verified against the codebase. The document correctly describes the policy flags enabled by each preset and their rationale.

Reviewed and updated in Round 1 by AI 2 on 2025-09-12 15:01 +02:00
