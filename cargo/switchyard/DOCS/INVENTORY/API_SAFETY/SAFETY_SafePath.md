# SafePath (capability-scoped paths)

- Category: Safety
- Maturity: Silver

## Summary

`SafePath` ensures mutating APIs operate within a caller-provided root, preventing path traversal and root escape. Aligns with SPEC Reproducible v1.1: “SafePath for all mutating APIs”.

## Implementation

- Core type: `cargo/switchyard/src/types/safepath.rs::SafePath`
  - Validates components (rejects `ParentDir`, normalizes `CurDir`), preserves `rel()` under a fixed `root`.
- Usage:
  - API types and handlers use `SafePath` for sources/targets:
    - `cargo/switchyard/src/api/plan.rs` builds actions with `SafePath`.
    - `cargo/switchyard/src/api/apply/handlers.rs` receives `Action::{EnsureSymlink, RestoreFromBackup}` with `SafePath`.
  - Filesystem atoms accept `SafePath`:
    - `cargo/switchyard/src/fs/swap.rs::replace_file_with_symlink(source: &SafePath, target: &SafePath, ...)`.
    - `cargo/switchyard/src/fs/restore.rs::restore_file(target: &SafePath, ...)`.

## Wiring Assessment

- Entry points: `Switchyard::plan` produces `Plan` with `SafePath` members; `preflight` and `apply` operate on those.
- Adapters/Policy: OwnershipOracle and Policy operate on `SafePath`-wrapped targets.
- Stages: Preflight checks and Apply mutations always go through `SafePath`→absolute via `as_path()`.
- Conclusion: wired correctly; no mutating path API accepts raw `Path` for targets.

## Evidence and Proof

- Unit tests: `cargo/switchyard/src/types/safepath.rs` (`rejects_dotdot`, `accepts_absolute_inside_root`, etc.).
- Apply/Preflight tests indirectly prove integration using `SafePath` construction.

## Gaps and Risks

- No serialization/deserialization for `SafePath` yet (not required internally).
- Does not yet enforce per-target SafePath roots heterogeneously in a plan (plan-level root assumed).

## Next Steps to Raise Maturity

- Add property tests for normalization/idempotence.
- Integrate schema validation if/when `SafePath` crosses FFI/CLI boundaries.

## Related

- SPEC v1.1 (SafePath requirement).
- `cargo/switchyard/src/fs/paths.rs::is_safe_path()` (auxiliary guard).
