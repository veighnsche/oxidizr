# Switchyard Implementation Inventory

This folder tracks one entry per implemented feature across Safety, UX/DX, and Infra. Each entry includes a maturity rating and proofs with code citations. Use `TEMPLATE.md` for new entries.

## Maintenance and Contributions

Use this inventory as a living document. Keep entries up-to-date as code changes land.

- Observations
  - Add short notes under an entry’s "Observations log" with date, author, and a brief description of what was discovered (e.g., edge cases, flaky behavior, environment caveats).
  - Example: "2025-09-13 — vince — EXDEV on btrfs observed in container; degraded path ok."

- Updates
  - When code or behavior changes, update the entry’s code citations and any affected fields (policy knobs, emitted facts, error mapping).
  - If maturity changes, bump the tier and briefly justify in "Change history" with a link to the PR.

- Maintenance cadence
  - Each entry carries "Owner(s)" and "Last reviewed" metadata. Target regular reviews (e.g., monthly or per release).
  - If ownership changes, update the entry header.

- PR workflow
  - If your PR modifies code paths cited by an entry, update that entry in the same PR.
  - Checklist: update code references; run tests; add or update goldens; update maturity if warranted; append a dated observation if adding nuance.

- Living inventory guidance
  - Prefer precise repo-relative paths for code references (e.g., `cargo/switchyard/src/fs/mount.rs`).
  - Keep entries concise; link to SPEC/ADRs for depth. Use the maintenance checklist in `TEMPLATE.md`.

### Licensing Inventory

Document and maintain licensing information as a dedicated inventory entry to ensure compliance and attribution.

- Where
  - Create `GOV_Licensing_Inventory.md` in this folder (or `docs/licensing/` if you prefer a separate area) using `TEMPLATE.md` as a base.

- What to capture
  - Project license(s): reference and link to `LICENSE`, `LICENSE-MIT`, etc. at repo root.
  - SPDX expression for the project if applicable (e.g., in `Cargo.toml` or `package metadata`).
  - Dependency license summary: generate and attach reports (store under `golden-diff/` or `docs/licensing/`).
  - Exceptions/allowlist/denylist and decisions with rationale.
  - NOTICE/attribution requirements and third-party notices file path(s).
  - SBOM location and format (SPDX/CycloneDX), generation commands, and version.

- How to maintain
  - Trigger: on dependency changes (lockfile updates), releases, or policy changes.
  - Review cadence: at least per release; update "Last reviewed" metadata.
  - Add a dated entry to the "Observations log" when noteworthy variances are found (e.g., new license types).

- Example commands
  - cargo-deny (policy-driven checks)
    - Install: `cargo install cargo-deny`
    - Run: `cargo deny check licenses`
  - cargo-about (generate human-readable inventory)
    - Install: `cargo install cargo-about`
    - Init template: `cargo about init about.hbs`
    - Generate: `cargo about generate about.hbs > docs/licensing/THIRD_PARTY_NOTICES.md`
  - SBOM generators (optional)
    - CycloneDX: `cargo install cargo-cyclonedx && cargo cyclonedx -o docs/licensing/sbom.cdx.json`
    - SPDX: `cargo install spdx-rs` (or use CI action) and export to `docs/licensing/sbom.spdx.json`

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
  - [Ownership and provenance](SAFETY_Ownership_and_Provenance.md) — Silver
  - [Attestation](SAFETY_Attestation.md) — Bronze
  - [Facts schema validation](SAFETY_Facts_Schema_Validation.md) — Bronze
  - [Node hazards: SUID/SGID and hardlinks](SAFETY_Node_Hazards_SUID_SGID_and_Hardlinks.md) — Silver
  - [Preservation capabilities probe](SAFETY_Preservation_Capabilities_Probe.md) — Silver

- UX / DX
  - [Preflight YAML exporter](UX_Preflight_YAML.md) — Bronze
  - [Adapters and extensibility](DX_Adapters_and_Extensibility.md) — Bronze
  - [Developer ergonomics](DX_Dev_Ergonomics.md) — Silver
  - [Traceability tools](DX_Traceability_Tools.md) — Bronze
  - [Best-effort conventions and catalog](DX_Best_Effort_Conventions.md) — Bronze

- Infra / Ops
  - [Rescue profile verification](INFRA_Rescue_Profile_Verification.md) — Silver
  - [Mount checks (rw+exec)](INFRA_Mount_Checks.md) — Silver
  - [Backup retention and prune](INFRA_Backup_Retention_Prune.md) — Bronze
  - [JSONL file logging sink](INFRA_JSONL_File_Logging.md) — Bronze
  - [Smoke tests and auto-rollback](INFRA_Smoke_Tests_Auto_Rollback.md) — Silver
  - [Operational bounds](INFRA_Operational_Bounds.md) — Bronze
  - [Golden fixtures and CI gates](INFRA_Golden_Fixtures_and_CI_Gates.md) — Bronze

Conventions:

- Maturity tiers follow `PLAN/90-implementation-tiers.md` (Bronze → Platinum).
- All code references use repository-relative paths, e.g., `cargo/switchyard/src/...`.
- Keep entries short but precise; link to SPEC/ADRs for deeper context.
