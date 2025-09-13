# CLIPPY Remediation Plan: policy/gating.rs::evaluate_action

- Lint: clippy::too_many_lines (175/100)

## Proof (code reference)

```rust
pub(crate) fn evaluate_action(
    policy: &Policy,
    owner: Option<&dyn DebugOwnershipOracle>,
    act: &Action,
) -> Evaluation {
    // ... 175 LOC total
}
```

Source: `cargo/switchyard/src/policy/gating.rs`

## Goals

- Split per-action evaluators and deduplicate common checks.

## Proposed helpers

- `fn eval_ensure_symlink(policy: &Policy, owner: Option<&dyn DebugOwnershipOracle>, source: &SafePath, target: &SafePath) -> Evaluation`
- `fn eval_restore_from_backup(policy: &Policy, owner: Option<&dyn DebugOwnershipOracle>, target: &SafePath) -> Evaluation`
- Common helpers:
  - `mount_rw_exec_check(p: &Path)`
  - `immutable_check(p: &Path)`
  - `hardlink_risk_check(policy: &Policy, p: &Path)`
  - `suid_sgid_risk_check(policy: &Policy, p: &Path)`
  - `source_trust_check(policy: &Policy, src: &Path)`
  - `scope_allow_forbid_check(policy: &Policy, target: &Path)`

## Architecture alternative (preferred): Checklist pipeline

Make gating declarative by composing small checks into a pipeline per action kind. Each check returns a normalized output consumed to build `Evaluation`.

- Define:

  ```rust
  struct CheckOutput { stop: Option<String>, note: Option<String> }
  trait GateCheck { fn run(&self) -> CheckOutput }
  ```

- Provide concrete checks (wrapping `preflight::checks`): `MountRwExecCheck`, `ImmutableCheck`, `HardlinkRiskCheck`, `SuidSgidRiskCheck`, `SourceTrustCheck`, `ScopeCheck`.
- For each action kind, assemble a `Vec<Box<dyn GateCheck>>` based on policy and action fields, run them, and fold into `Evaluation`.
- Benefits: eliminates repeated code, easier to test and extend per policy preset.

### Updated Implementation TODOs (preferred)

- [ ] Define `CheckOutput` and `GateCheck` in `policy/gating.rs` or a sibling `gating/checks.rs`.
- [ ] Implement concrete check structs that wrap existing functions in `preflight/checks.rs`.
- [ ] Rework `evaluate_action` to build pipelines for `EnsureSymlink` and `RestoreFromBackup` and fold outputs.
- [ ] Ensure STOP/notes text remains byte-for-byte compatible to avoid regressions.

## Implementation TODOs (fallback: helper split only)

- [ ] Route match arms to `eval_*` functions.
- [ ] Implement and reuse the common helpers to DRY repeated logic.
- [ ] Preserve STOP/notes behavior exactly.

## Acceptance criteria

- [ ] Function < 100 LOC; stops/notes unchanged for sample inputs.
- [ ] Clippy clean for this function.

## Test & verification notes

- Add small tests around individual helpers to ensure stable STOP/note outputs for representative paths.
