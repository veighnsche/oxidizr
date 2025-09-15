# Apply

- Atomic symlink swap with backup/restore.
- Facts: per-action attempt/result and final summary.
- Degraded EXDEV support when policy allows, records `degraded=true`.

Citations:
- `cargo/switchyard/src/api/apply/mod.rs`
- `cargo/switchyard/src/fs/{atomic.rs,swap.rs}`
