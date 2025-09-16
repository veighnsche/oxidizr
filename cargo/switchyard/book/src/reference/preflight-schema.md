# Preflight Schema

- Rows contain (SPEC §4):
  - `action_id`, `path`
  - `current_kind` ∈ {missing, file, dir, symlink}
  - `planned_kind` ∈ {symlink, restore_from_backup, skip}
  - `policy_ok: bool`
  - `provenance` (optional): `uid`, `gid`, `pkg`
  - `notes: string[]`
  - `preservation` (optional): `owner`, `mode`, `timestamps`, `xattrs`, `acls`, `caps`
  - `preservation_supported: bool`

- Deterministic ordering: rows ordered by (`path`, `action_id`) to stabilize goldens.
- Exporter for YAML lives under `preflight::yaml`.

Citations:
- `src/api/preflight/mod.rs`
- `src/preflight/yaml.rs`
- `SPEC/SPEC.md`
