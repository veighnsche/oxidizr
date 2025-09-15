# Cross-filesystem (EXDEV)

- Atomic rename across filesystems is not possible for symlinks.
- When policy allows, degrade to unlink+symlink and mark `degraded=true`.

Citations:
- `cargo/switchyard/src/fs/atomic.rs`
- `cargo/switchyard/src/api/apply/handlers.rs`
- `cargo/switchyard/SPEC/SPEC.md`
