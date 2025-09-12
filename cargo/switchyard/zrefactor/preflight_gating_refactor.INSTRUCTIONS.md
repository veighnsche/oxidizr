# Preflight Orchestration + Policy-owned Gating — Actionable Steps (breaking)

Do these changes to centralize gating in policy and keep preflight thin.

1) Implement policy-owned evaluator

- File: `src/policy/gating.rs` (or `src/policy/evaluator.rs`)
  - Define:

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

  - Expose:

    ```rust
    pub fn evaluate_action(policy: &Policy, owner: Option<&dyn OwnershipOracle>, act: &Action) -> ActionEvaluation
    ```

  - Interpret grouped policy knobs (Scope, Risks, Durability, ApplyFlow, Rescue).
  - Use shared helpers for checks (mount rw+exec, SUID/SGID, hardlinks, immutable, source trust, scope, preservation).

2) Refactor preflight to consume evaluator

- File: `src/api/preflight/mod.rs`
  - For each `Action` in the `Plan`, call `policy::gating::evaluate_action(..)`.
  - Emit rows via logging facade (`StageLogger`) only; do not build JSON directly.
  - Remove duplicate SUID/SGID check.
  - Replace hard-coded "/usr" with iteration over `policy.apply.extra_mount_checks`.

3) Apply-time gating uses the same evaluator

- Files: `src/api/apply/*.rs`
  - Before any mutation, call `policy::gating::evaluate_action(..)`.
  - Enforce `override_preflight`: if false and `stops` non-empty, abort.

4) Move/align low-level helpers

- Prefer reusable helpers under `src/fs/**` or `src/types/**`.
- Keep only generic stateless helpers in `src/preflight/checks.rs` if needed by multiple modules.

5) Tests

- Unit: cover evaluator cases — SUID/SGID stop/warn/allow, hardlink hazard stop/warn, source trust variants, scope allow/forbid, preservation requirement, extra mount checks.
- Integration: parity test asserts preflight and apply decisions are identical for the same plan.

6) CI guardrails

- Grep forbid duplicating gating logic outside `src/policy/gating.rs`.
- Grep forbid ad-hoc preflight checks in `src/api/**` (must call the evaluator).

7) Cleanups

- /// remove this file: `src/policy/gating.rs` (legacy duplicate, if separate copy exists) and keep only the new evaluator.
- Ensure `src/api/preflight/mod.rs` contains no business rules, only orchestration + logging.
