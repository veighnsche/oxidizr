# CLIPPY Remediation Plan: fs/restore/engine.rs::restore_impl

- Lint: clippy::too_many_lines (107/100)

## Proof (code reference)

```rust
pub fn restore_impl(
    target: &SafePath,
    sel: SnapshotSel,
    opts: &RestoreOptions,
) -> std::io::Result<()> {
    // ... 107 LOC total
}
```

Source: `cargo/switchyard/src/fs/restore/engine.rs`

## Goals

- Split logic by `prior_kind` and selection; preserve best-effort and integrity behavior.

## Architecture alternative (preferred): RestorePlanner (plan→execute)

Shift `restore_impl` to a two-phase model: planning (pure) and execution (I/O). This simplifies control flow and makes testing easier.

- Define:

  ```rust
  enum RestoreAction {
      Noop,
      FileRename { backup: PathBuf, mode: Option<u32> },
      SymlinkTo { dest: PathBuf, cleanup_backup: bool },
      EnsureAbsent,
      LegacyRename { backup: PathBuf },
  }
  struct RestorePlanner;
  impl RestorePlanner {
      fn plan(target: &Path, sel: SnapshotSel, opts: &RestoreOptions) -> std::io::Result<(Option<PathBuf>, Option<Sidecar>, RestoreAction)> { /* select, read sidecar, idempotence, derive action */ }
  }
  ```

- Execution maps `RestoreAction` variants to existing `steps::*` helpers.
- Best-effort and integrity behaviors remain identical; idempotence yields `Noop`.

### Implementation plan (preferred, granular)

- [ ] Implement `RestorePlanner::plan` using current selection + sidecar reading + idempotence logic.
- [ ] Implement `execute(action)` that calls `steps::{restore_file_bytes, restore_symlink_to, legacy_rename, ensure_absent}` as appropriate.
- [ ] Refactor `restore_impl` to `let (_, _, action) = RestorePlanner::plan(...)?; if opts.dry_run { return Ok(()); } execute(action)`.
- [ ] Add unit tests over `plan` to validate scenarios (file, symlink, none, other; with/without backup; best-effort; integrity mismatch).

## Acceptance criteria

- [ ] Function < 100 LOC.
- [ ] Behavior preserved (dry-run, best-effort, integrity verification).
- [ ] Clippy clean for this function.

## Test & verification notes

- Add targeted unit tests for file and symlink restore branches; exercise best-effort paths.
