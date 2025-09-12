# Policy Refactor — Actionable Steps (breaking)

> STATUS: Not landed in src/ (as of 2025-09-12 23:16:50 +02:00). `src/policy/config.rs` still uses flat fields; `src/policy/gating.rs` contains a plan-level `gating_errors(...)` helper, but no typed `evaluate_action(...)` consumed by API stages. Keep PRs refactor-only.

Do these steps to replace the flat `Policy` with typed groups and central gating.

1) Create new typed groups and enums
- File: `src/policy/types.rs`
  - Enums:
    - `RiskLevel { Stop, Warn, Allow }`
    - `ExdevPolicy { Fail, DegradedFallback }`
    - `LockingPolicy { Required, Optional }`
    - `SmokePolicy { Off, Require { auto_rollback: bool } }`
    - `PreservationPolicy { Off, RequireBasic }`
    - `SourceTrustPolicy { RequireTrusted, WarnOnUntrusted, AllowUntrusted }`
  - Groups (structs):
    - `Scope { allow_roots: Vec<PathBuf>, forbid_paths: Vec<PathBuf> }`
    - `Rescue { require: bool, exec_check: bool, min_count: usize }`
    - `Risks { suid_sgid: RiskLevel, hardlinks: RiskLevel, source_trust: SourceTrustPolicy, ownership_strict: bool }`
    - `Durability { backup_durability: bool, sidecar_integrity: bool, preservation: PreservationPolicy }`
    - `ApplyFlow { exdev: ExdevPolicy, override_preflight: bool, best_effort_restore: bool, extra_mount_checks: Vec<PathBuf>, capture_restore_snapshot: bool }`
    - `Governance { locking: LockingPolicy, smoke: SmokePolicy, allow_unlocked_commit: bool }`
    - `Backup { tag: String }`

2) Reimplement `Policy` with groups (remove legacy fields)
- File: `src/policy/config.rs`
  - Define:
    ```rust
    pub struct Policy {
        pub scope: Scope,
        pub rescue: Rescue,
        pub risks: Risks,
        pub durability: Durability,
        pub apply: ApplyFlow,
        pub governance: Governance,
        pub backup: Backup,
    }
    ```
  - Remove legacy flat boolean fields completely.
  - Provide constructors for profiles in `profiles.rs` and a builder in `builder.rs`.

3) Add ergonomic profiles and builder
- File: `src/policy/profiles.rs`
  - Implement `Policy::production()`, `Policy::coreutils_switch()`, `Policy::permissive_dev()`.
- File: `src/policy/builder.rs`
  - Implement `PolicyBuilder::production()` with nested scopes: `.scope()`, `.risks()`, `.apply()`, `.governance()`.

4) Implement policy-owned gating evaluator
- File: `src/policy/gating.rs` (or `src/policy/evaluator.rs`)
  - Define a typed result:
    ```rust
    pub struct ActionEvaluation {
        pub warnings: Vec<String>,
        pub stops: Vec<String>,
        pub notes: Vec<String>,
        pub policy_ok: bool,
        pub provenance: Option<serde_json::Value>,
        pub preservation: Option<serde_json::Value>,
        pub preservation_supported: Option<bool>,
    }
    ```
  - Expose: `pub fn evaluate_action(policy: &Policy, owner: Option<&dyn OwnershipOracle>, act: &Action) -> ActionEvaluation`.
  - Apply checks via shared helpers (mount rw+exec, SUID/SGID, hardlinks, immutable, source trust, scope, preservation).

5) Update preflight to consume policy evaluator
- File: `src/api/preflight/mod.rs`
  - Replace inlined checks with calls to `policy::gating::evaluate_action(..)` for each `Action`.
  - Emit rows via logging facade only (no JSON assembly).
  - Ensure mount checks use `policy.apply.extra_mount_checks` (no hard-coded "/usr").

6) Update apply-stage gating to use evaluator
- Files: `src/api/apply/*.rs`
  - Before mutating, call `policy::gating::evaluate_action(..)`.
  - Enforce `override_preflight`: if false and `stops` non-empty, abort.

7) Replace legacy API and imports
- Remove all usages of legacy flat fields.
- Update modules to import from `policy::{types, profiles, builder, gating}`.

8) Tests
- Unit tests: enums and groups behaviors; evaluator result combinations.
- Integration: preflight vs apply parity; `extra_mount_checks` applied on Restore.

9) CI guardrails
- Grep forbid references to removed legacy fields in `Policy`.
- Grep forbid duplicating gating logic outside `src/policy/gating.rs`.

10) Cleanups
- Add module docs and rustdoc to new types.
- /// remove this file: `src/policy/checks.rs` (if it is a legacy compat shim)
- /// remove legacy presets if duplicated by `profiles.rs`.

---

## Meta

- Scope: Replace flat Policy with grouped types/enums; centralize gating decisions
- Status: Breaking allowed (pre-1.0)
- Index: See `zrefactor/README.md`

## Related

- API usage and wiring: `zrefactor/api_refactor.INSTRUCTIONS.md`
- Preflight/apply orchestrators consuming evaluator: `zrefactor/preflight_gating_refactor.INSTRUCTIONS.md`
- Logging facade integration (fields/envelope): `zrefactor/logging_audit_refactor.INSTRUCTIONS.md`
- Cohesion/guardrails: `zrefactor/responsibility_cohesion_report.md`
- Removals and registry: `zrefactor/backwards_compat_removals.md`, `zrefactor/removals_registry.md`
