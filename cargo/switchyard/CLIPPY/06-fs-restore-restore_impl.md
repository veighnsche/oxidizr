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

## Proposed helpers

- `fn select_backup_pair(target: &Path, sel: SnapshotSel, tag: &str) -> Option<(Option<PathBuf>, PathBuf)>`
- `fn early_exit_if_idempotent(target: &Path, sc: &Sidecar) -> bool`
- `fn restore_file_bytes_kind(target: &Path, backup: &Path, mode_oct: Option<u32>) -> std::io::Result<()>`
- `fn restore_symlink_kind(target: &Path, dest: &Path, backup_opt: Option<&Path>) -> std::io::Result<()>`
- `fn legacy_rename_or_best_effort(target: &Path, backup: Option<&Path>, force: bool) -> std::io::Result<()>`

## Implementation TODOs

- [ ] Extract selection logic and sidecar read to helpers.
- [ ] Factor `prior_kind` arms into dedicated functions (file, symlink, none, other).
- [ ] Keep hash verification and best-effort behavior identical.

## Acceptance criteria

- [ ] Function < 100 LOC.
- [ ] Behavior preserved (dry-run, best-effort, integrity verification).
- [ ] Clippy clean for this function.

## Test & verification notes

- Add targeted unit tests for file and symlink restore branches; exercise best-effort paths.
