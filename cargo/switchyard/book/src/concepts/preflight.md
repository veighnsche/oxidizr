# Preflight

- Emits one row per action and a summary.
- Gates (fail‑closed by default unless policy overrides):
  - Mount checks: parent mounts must be `rw+exec`; ambiguous → STOP.
  - Immutability: targets on immutable FS/attrs cause STOP.
  - Source trust & ownership: enforce `ownership_strict` when enabled; STOP if oracle missing or mismatched.
  - Preservation capabilities: verify support for owner/mode/timestamps/xattrs/ACLs/caps; STOP if required but unsupported.
  - Node hazards: SUID/SGID and hardlinks detected → STOP by default (policy can downgrade to warn/allow).
  - Sidecar integrity: if required and a `payload_hash` is present, restore paths must verify hash.
- Rescue: verify BusyBox or a deterministic GNU subset when `rescue.require=true`; STOP if missing.

Output shape
- Rows include `action_id`, `path`, `current_kind`, `planned_kind`, `policy_ok`, optional `provenance`, `preservation`, `notes`.
- A summary event reports STOP reasons and maps to exit taxonomy (e.g., `E_POLICY`).

Citations:
- `src/api/preflight/mod.rs`
- `src/preflight/checks.rs`
- `SPEC/SPEC.md`
- Inventory: `INVENTORY/35_FS_Mount_Checks.md`, `INVENTORY/45_Policy_Node_Hazards_SuidSgid_Hardlinks.md`, `INVENTORY/40_Policy_Ownership_and_Provenance.md`, `INVENTORY/30_Infra_Rescue_Profile_Verification.md`
