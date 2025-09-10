# PROFILES LAYOUT & ATOMIC FLIP

## 1) Scope (What this project changes)

- Explicit boundaries and non-goals.
  - Changes the symlink management model from N-per-target swaps to a single profile pointer flip.
  - Does not remove existing emergency restore logic (`src/symlink/ops.rs::restore_file`).
  - Does not change applet discovery rules in experiments, only the final linking topology.
- Source files/modules likely touched (paths + key symbols).
  - `src/symlink/ops.rs::{atomic_symlink_swap, restore_file, fsync_parent_dir}`
  - `src/experiments/util.rs::{create_symlinks, restore_targets, resolve_usrbin}`
  - `src/experiments/coreutils.rs::{enable, disable, discover_applets, list_targets}`
  - `src/experiments/findutils.rs::{enable, disable, discover_applets, list_targets}`
  - `src/state/mod.rs::{set_enabled, save_state, load_state}`
  - `src/system/lock.rs` (single-instance lock during flips)
- User-visible behavior changes (if any).
  - Profile-based layout under `/usr/lib/oxidizr-arch/profiles/{gnu,uutils}/bin` with a single `active` symlink.
  - Flips between GNU and uutils are atomic by updating `.../active` via `renameat(2)`.

## 2) Rationale & Safety Objectives

- Why this is needed (short).
  - Reduces blast radius: avoid partially updated systems when linking dozens of applets.
- Safety invariants this project enforces (atomicity, rollback, minimal surface, auditability).
  - Atomicity: one `renameat` to flip `active` pointer (same-directory rename; `src/symlink/ops.rs::atomic_symlink_swap`).
  - Rollback: flip back to previous profile by renaming `active` to prior target; emergency per-target restore remains.
  - Minimal surface: fewer moving parts at flip time (single pointer instead of N symlinks).
  - Auditability: single per-operation record with clear before/after active profile.
- Overkill → Lean replacement summary (if applicable).
  - Replace: "Transaction rollback across multi-target operations" with "Profiles layout + single active pointer flip".

## 3) Architecture & Design

- High-level approach (one paragraph).
  - Introduce a profiles directory with two subprofiles (`gnu`, `uutils`) each containing a `bin/` of applet entry points. `/usr/bin/<applet>` will be a stable symlink to `.../active/bin/<applet>`. Enabling/disabling experiments updates which profile is pointed to by the `active` symlink via an atomic `renameat(2)` flip. Experiments still compute the set of applets, but actual commit is a single pointer switch.
- Data model & structures (types/fields; JSON examples if relevant).
  - State extension (if needed): `state.json` may include `active_profile: "gnu"|"uutils"` alongside `enabled_experiments` and `managed_targets` (see `src/state/mod.rs::State`).
- Control flow: ASCII diagram of phases (Staging → Validation → Commit → Verify → Rollback).
  - Staging: Build/refresh `profiles/{gnu,uutils}/bin/<applet>` trees (no effect on `/usr/bin`).
  - Validation: Verify sources via trust checks; ensure all targets resolvable.
  - Commit: `renameat` `.../active.tmp` → `.../active` (atomic flip).
  - Verify: `readlink` a sample of `/usr/bin/<applet>` to confirm `.../active/...`.
  - Rollback: flip back to prior profile via one `renameat`.
- Public interfaces (function signatures, CLI flags, env vars).
  - CLI: `oxidizr-arch enable/disable` continue; internal flip replaced.
  - Optional: `oxidizr-arch profile --set {gnu|uutils}` for direct pointer flip.
- Filesystem layout (if relevant), with example paths.
  - `/usr/lib/oxidizr-arch/profiles/gnu/bin/ls`
  - `/usr/lib/oxidizr-arch/profiles/uutils/bin/ls`
  - `/usr/lib/oxidizr-arch/active -> /usr/lib/oxidizr-arch/profiles/gnu`
  - `/usr/bin/ls -> /usr/lib/oxidizr-arch/active/bin/ls`

## 4) Failure Modes & Guarantees

- Enumerate failure cases and how they’re detected.
  - Missing profile tree: detect before commit; error and abort.
  - Permission/immutable flags: surfaced via `src/system/fs_checks.rs::{check_immutable, ensure_mount_rw_exec}`.
- Rollback strategy and exact recovery commands.
  - `mv -T active active.prev && mv -T <old> active` or equivalent atomic rename pair.
  - Emergency: `src/experiments/util.rs::restore_targets` and `src/symlink/ops.rs::restore_file` per target.
- Idempotency rules.
  - Flipping to the currently active profile is a no-op; validation still runs.
- Concurrency/locking notes.
  - Use single-instance process lock `src/system/lock.rs` to serialize flips.

## 5) Preflight & Post-Change Verification

- Preflight checks (conditions to proceed; how failures are reported).
  - Validate sources exist for all applets; ownership and mount exec checks.
  - Ensure profile tree built and `active.tmp` path writable.
- Post-change smoke tests and success criteria.
  - Run a minimal suite (`ls --version`, `cp --version`, `find --version`) against `/usr/bin/*` after flip.

## 6) Observability & Audit

- Structured log fields (key=value / JSONL schema).
  - `event=profile_flip`, `from_profile`, `to_profile`, `duration_ms`.
- When and how to produce per-operation artifacts (e.g., `audit-<op_id>.jsonl`, `.sig`).
  - Emit per-operation JSONL and detached signature per the attestation plan.
- Provenance and selective hashing policy (if applicable).
  - Hash only changed applets or unowned sources during the flip.

## 7) Security & Policy

- Permissions/ownership/labels/xattrs handling (split: file vs symlink).
  - Preserve metadata for regular-file backups; symlinks preserve linkness + target.
- Environment sanitization (`LC_ALL`, PATH pinning).
  - Normalize `LC_ALL=C` for external probes.
- Supply-chain rules (repo-first, AUR opt-in).
  - Enforced by package worker policies.

## 8) Migration Plan

- From current state → new state (step-by-step).
  1. Create `profiles/{gnu,uutils}/bin` and populate entries.
  2. Update `/usr/bin/<applet>` to point to `.../active/bin/<applet>`.
  3. Initialize `active` to current provider.
- Backward compatibility and safe rollback to old model.
  - Old per-target backups remain valid; `restore_file` continues to function.
- One-liner “escape hatch” for operators (e.g., PATH override).
  - `export PATH=/usr/lib/oxidizr-arch/profiles/gnu/bin:$PATH`

## 9) Testing Strategy

- Unit tests, integration/E2E cases, fault injection.
  - Flip tests: ensure atomic swap and no partial states.
  - Fault injection mid-flip; verify rollback works by pointer re-flip.
- Determinism checks.
  - Stable ordering of build steps and logs.
- CI hooks and gating.
  - Integrate with `test-orch/` container runner.

## 10) Acceptance Criteria (Must be true to ship)

- Single `active` pointer flip replaces N per-target writes.
- Flip and revert are atomic and idempotent.
- Smoke tests pass post-flip; audit artifacts present and verifiable.

## 11) Work Breakdown & Review Checklist

- Phased tasks (small PRs).
  - Define filesystem layout and scaffolding.
  - Wire experiments to populate profile trees.
  - Implement atomic pointer flip and verification.
  - Remove per-target flip from enable path (keep restore).
- Reviewer checklist (red flags to look for).
  - Any per-target writes during commit; missing fsync/rename semantics.
- Estimated diff budget (lines/files) to keep reviews humane.
  - Medium (5–10 files; ~300–500 LOC).

## 12) References (Repo evidence only)

- Paths + symbols supporting this plan (no external links).
  - `src/symlink/ops.rs::{atomic_symlink_swap, restore_file, fsync_parent_dir}`
  - `src/experiments/util.rs::{create_symlinks, restore_targets}`
  - `src/experiments/coreutils.rs::{enable, disable, discover_applets}`
  - `src/experiments/findutils.rs::{enable, disable, discover_applets}`
  - `src/state/mod.rs::{set_enabled, save_state}`
  - `src/system/fs_checks.rs::{ensure_mount_rw_exec, check_immutable}`
- TODO_LIST_V2.md items covered by this project.
  - "Profiles layout + single active pointer flip"
  - "Preflight plan and diff"
  - "Automatic smoke tests and rollback triggers"
  - "Determinism guardrails"
