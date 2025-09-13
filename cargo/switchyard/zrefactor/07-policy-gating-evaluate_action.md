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

### Implementation plan (preferred, granular)

- [ ] Create module layout
  - [ ] Add `src/policy/gating/checks.rs` (or nested module) to host types and concrete checks; re-export from `gating.rs`.
  - [ ] Define types:

    ```rust
    pub(crate) struct CheckOutput { pub stop: Option<String>, pub note: Option<String> }
    pub(crate) trait GateCheck { fn run(&self) -> CheckOutput }
    ```

- [ ] Implement concrete check structs wrapping `preflight::checks`
  - [ ] `MountRwExecCheck { path: PathBuf }` → uses `ensure_mount_rw_exec`; on Err(e) set `stop=Some(..)`, `note=Some(..)` with same wording.
  - [ ] `ImmutableCheck { path: PathBuf }` → uses `check_immutable`; on Err(e) produce identical message text.
  - [ ] `HardlinkRiskCheck { policy: Policy, path: PathBuf }` → uses `check_hardlink_hazard`; map to Stop/Note per `policy.risks.hardlinks`.
  - [ ] `SuidSgidRiskCheck { policy: Policy, path: PathBuf }` → uses `check_suid_sgid_risk`; map to Stop/Note per `policy.risks.suid_sgid`.
  - [ ] `SourceTrustCheck { policy: Policy, source: PathBuf }` → uses `check_source_trust` with `force` per policy; Stop/Note wording preserved.
  - [ ] `ScopeCheck { policy: Policy, target: PathBuf }` → checks `allow_roots` and `forbid_paths` with identical messages.
- [ ] Build per-action pipelines in `evaluate_action`
  - [ ] For `EnsureSymlink { source, target }`: assemble Vec<Box<dyn GateCheck>> with mount checks (extra + target), immutable, hardlink risk (target), suid/sgid (target), source trust (source), scope check (target), and strict ownership check using `owner` when `ownership_strict`.
  - [ ] For `RestoreFromBackup { target }`: assemble appropriate checks (no source trust).
  - [ ] Run all checks; fold outputs into `Evaluation { policy_ok, stops, notes }` while preserving order and exact text.
- [ ] Ownership strict handling
  - [ ] Keep inline check for ownership using `owner.owner_of(target)` with identical stop/note messages when strict.
- [ ] Invariants and compatibility
  - [ ] Message strings (STOPs and notes) must remain byte-for-byte identical to current outputs.
  - [ ] Evaluation ordering of stops/notes preserved relative to present implementation.
  - [ ] Function line count < 100 after refactor.
- [ ] Tests
  - [ ] Unit-test each GateCheck with synthetic success/failure paths (mock filesystem where feasible).
  - [ ] Integration tests for `evaluate_action` for both action kinds and key policy permutations (hardlinks Stop/Warn/Allow, suid/sgid Stop/Warn/Allow, ownership_strict on/off).
  - [ ] Ensure aggregated stops/notes match current outputs for a set of golden inputs.

## Acceptance criteria

- [ ] Function < 100 LOC; stops/notes unchanged for sample inputs.
- [ ] Clippy clean for this function.

## Test & verification notes

- Add small tests around individual helpers to ensure stable STOP/note outputs for representative paths.
