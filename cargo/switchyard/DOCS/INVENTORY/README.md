# Switchyard Implementation Inventory

This folder tracks one entry per implemented feature across Safety, UX/DX, and Infra. Each entry includes a maturity rating and proofs with code citations. Use `TEMPLATE.md` for new entries.

## Index (Quick Reference)

- Safety
  - [SafePath (capability-scoped paths)](SAFETY_SafePath.md) — Silver
  - [Atomic symlink swap (TOCTOU-safe)](SAFETY_Atomic_Symlink_Swap.md) — Silver
  - [Policy gating and preflight](SAFETY_Policy_Gating_and_Preflight.md) — Silver
  - [Locking and concurrency](SAFETY_Locking_and_Concurrency.md) — Silver (with adapter)
  - [Backup and sidecar](SAFETY_Backup_and_Sidecar.md) — Silver
  - [Restore and rollback](SAFETY_Restore_and_Rollback.md) — Silver
  - [Determinism and redaction](SAFETY_Determinism_and_Redaction.md) — Silver
  - [Audit and logging](SAFETY_Audit_and_Logging.md) — Silver
  - [Exit codes taxonomy](SAFETY_Exit_Codes.md) — Silver

- UX / DX
  - [Preflight YAML exporter](UX_Preflight_YAML.md) — Bronze
  - [Adapters and extensibility](DX_Adapters_and_Extensibility.md) — Bronze
  - [Developer ergonomics](DX_Dev_Ergonomics.md) — Silver

- Infra / Ops
  - [Rescue profile verification](INFRA_Rescue_Profile_Verification.md) — Silver
  - [Mount checks (rw+exec)](INFRA_Mount_Checks.md) — Silver
  - [Backup retention and prune](INFRA_Backup_Retention_Prune.md) — Bronze
  - [JSONL file logging sink](INFRA_JSONL_File_Logging.md) — Bronze
  - [Smoke tests and auto-rollback](INFRA_Smoke_Tests_Auto_Rollback.md) — Silver

Conventions:

- Maturity tiers follow `PLAN/90-implementation-tiers.md` (Bronze → Platinum).
- All code references use repository-relative paths, e.g., `cargo/switchyard/src/...`.
- Keep entries short but precise; link to SPEC/ADRs for deeper context.
