# Preflight

- Emits one row per action and a summary.
- Gates: mount `rw+exec`, immutability, source trust, ownership, preservation.
- Rescue: verify BusyBox or GNU subset when required; STOP if missing.

Citations:
- `cargo/switchyard/src/api/preflight/mod.rs`
- `cargo/switchyard/src/preflight/checks.rs`
- `cargo/switchyard/SPEC/SPEC.md`
