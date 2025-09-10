# PROJECT_PLANS

This folder consolidates overlapping plans into leaner streams. Each stream maps to prior plans and cites concrete code entry points validated from the codebase audit. All streams integrate the adopted safety audit decisions; see `PROJECT_PLANS/SAFETY_DECISIONS_AUDIT.md`.

Streams:

- Stream A: Profiles & Atomic Flip + Canary + Smoke Tests + Backup Semantics
- Stream B: Preflight Plan & Compat Detectors + Dry-Run/Verbosity UX
- Stream C: Audit Attestation + Operator Docs (see also: Safety Decisions)
- Stream D: Supply Chain Policy (Repo-first/AUR opt-in) + Lock Wait UX
- Stream E: Dependency Footprint Trim
- Cross-cutting: Tests & CI Pipeline

## Principles: Reuse Existing Infrastructure

To keep the system lean and avoid building the same component twice, all work in Streams A–E must reuse the existing modules under `src/` and extend them in place:

- `src/symlink/ops.rs` is the sole authority for backup/restore and atomic swaps. Do not add a second symlink layer; call `replace_file_with_symlink` and `restore_file`.
- `src/logging/audit.rs` + `src/logging/init.rs` provide structured JSONL and human logs. Extend these (e.g., op buffer/signature) rather than creating a new logging sink.
- `src/system/worker/*.rs` is the only surface for system/package/PATH operations. Use `Worker.which()` for PATH lookups; do not call `which::which` directly.
- `src/system/worker/packages.rs` owns repository/AUR policy and lock-wait; add flags and progress there rather than building new execution paths.
- `src/experiments/*` and `src/experiments/util.rs` own discovery, planning, and link orchestration; add preflight/plan rendering here rather than duplicating logic elsewhere.
- `src/state/` and `src/ui/progress.rs` are reused for persistence and UX niceties.

Additions should be new functions or submodules under these existing files, not parallel implementations.

Mapping to original plans:

- A <= 1_PROFILES_&_ATOMIC_FLIP, 6_CANARY_SHELL_&_GNU_ESCAPE_PATH, 7_SMOKE_TESTS_&_AUTO_ROLLBACK, 3_METADATA_BACKUP_FILE_vs_SYMLINK
- B <= 2_PREFLIGHT_&_PLAN_DIFF, 8_COMPAT_MATRIX_&_PREFLIGHT_DETECTORS, (dry-run/verbosity slice of) 9_UX_CLI_VERBOSITY_POLICY_LOCK_WAIT
- C <= 4_AUDIT_ATTESTATION_SELECTIVE_HASHING, 12_DOCS_&_OPERATOR_PLAYBOOKS
- D <= 5_PACKAGE_SUPPLY_CHAIN_REPO_FIRST_AUR_OPTIN, (lock-wait slice of) 9_UX_CLI_VERBOSITY_POLICY_LOCK_WAIT
- E <= 10_DEPENDENCY_FOOTPRINT_TRIM
- Cross-cutting <= 11_TESTS_&_CI_PIPELINE

Codebase audit anchors (feasibility):

- CLI entry: `src/main.rs` initializes logging with dry-run gating via `OXIDIZR_DRY_RUN`, executes `cli::handle_cli()`
- Human/audit logging: `src/logging/init.rs`, `src/logging/audit.rs::{audit_event_fields, AUDIT_LOG_PATH}`
- Symlink ops (atomic renameat, backups): `src/symlink/ops.rs::{atomic_symlink_swap, restore_file, backup_path}`
- Experiments orchestration: `src/experiments/{coreutils.rs,findutils.rs}`, `src/experiments/util.rs::{create_symlinks, resolve_usrbin, restore_targets, log_applets_summary}`
- State/reporting: `src/state/mod.rs::{set_enabled, write_state_report}`
- Locking: `src/system/lock.rs::acquire`
- Package policy + lock wait: `src/system/worker/{packages.rs,distro.rs}`

Notes:

- These plans are implementation-ready slices; each cites concrete files/symbols for PR scoping.
- Safety decisions document: `PROJECT_PLANS/SAFETY_DECISIONS_AUDIT.md`.
- Original plans are left intact for provenance; this folder supersedes them conceptually.
