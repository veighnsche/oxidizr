# Preflight Schema

- Preflight rows contain: `action_id`, `path`, `current_kind`, `planned_kind`, `policy_ok`, optional `provenance`, `preservation`, and `restore_ready`.
- Exporter for YAML lives under `preflight::yaml`.

Citations:
- `cargo/switchyard/src/api/preflight/mod.rs`
- `cargo/switchyard/src/preflight/yaml.rs`
- `cargo/switchyard/SPEC/SPEC.md`
