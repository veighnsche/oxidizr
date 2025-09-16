# Rescue

- Verify BusyBox or a deterministic subset of GNU utilities when `rescue.require=true`.
- STOP if rescue profile unavailable.

Policy knobs
- `rescue.require: bool` — require a rescue profile before mutation.
- `rescue.exec_check: bool` — verify executability (x bits) on PATH.
- `rescue.min_count: usize` — minimal number of tools to consider acceptable.

Operator guidance
- Prefer BusyBox for compact, deterministic coverage; otherwise ensure a stable GNU subset (cp, mv, rm, ln, stat, readlink, sha256sum, sort, date, ls) with executability checks.
- Gate in preflight for production; treat missing rescue as `E_POLICY` and avoid overrides.

Citations:
- `cargo/switchyard/src/policy/rescue.rs`
- `cargo/switchyard/src/api/preflight/mod.rs`
- `cargo/switchyard/SPEC/SPEC.md`
- Inventory: `INVENTORY/30_Infra_Rescue_Profile_Verification.md`
