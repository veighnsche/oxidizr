# Reference: Preflight Schema

Fields per row:
- `action_id`, `path`, `current_kind`, `planned_kind`, `policy_ok`, `provenance?`, `preservation?`, `preservation_supported?`, `restore_ready?`

Summary:
- `rescue_profile`, optionally `error_id`, `exit_code`, and `summary_error_ids` on failure.

Citations:
- `cargo/switchyard/src/api/preflight/mod.rs`
- `cargo/switchyard/src/preflight/yaml.rs`
- `cargo/switchyard/SPEC/SPEC.md`
