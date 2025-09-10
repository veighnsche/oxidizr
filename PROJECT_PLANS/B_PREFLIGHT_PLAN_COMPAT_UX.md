# Stream B — Preflight Plan & Compat Detectors + Dry-Run/Verbosity UX

## 1) Scope

- Deterministic preflight that computes and renders the full plan without mutation.
- Small compatibility matrix detectors integrated into preflight.
- UX policy: dry-run-first posture and cleaner human INFO logs.

Touched modules (validated):

- `src/experiments/util.rs::{create_symlinks, log_applets_summary}` (plan builder/no-op mode)
- `src/experiments/*::{enable, list_targets, discover_applets}`
- `src/system/fs_checks.rs` (mount/immutable checks; add trust checks if needed)
- `src/cli/{parser.rs, handler.rs}` (add `--preflight`, adjust defaults)
- `src/logging/init.rs` (human formatter levels per `VERBOSITY.md`)
- New: `assets/compat_matrix.json` and a small scanner (e.g., `src/compat/mod.rs`)

## Reuse existing infrastructure

Implement preflight by extending existing modules, not by creating parallel logic:

- Build the plan via shared discovery in `src/experiments/*` and render through helpers in `src/experiments/util.rs`.
- Add CLI flags and orchestration only in `src/cli/{parser.rs,handler.rs}`; no separate binaries or entrypoints.
- Use `audit_event_fields` from `src/logging/audit.rs`; do not add a new logging sink.
- Perform PATH checks using `Worker.which()`; do not call `which::which` directly.
- Reuse `src/system/fs_checks.rs` for mount/immutable/trust checks.

## 2) Rationale & Safety Objectives

- Show users exactly what will change before commit.
- Fail-closed for risky flags/semantics for top applets (matrix driven).

## 3) Architecture & Design

- Build a `PreflightItem { target, current_kind, current_dest?, planned_kind, planned_dest?, policy_ok }` for each applet.
- Rendering in table form for human logs; embed plan rows in audit (`AuditFields.artifacts` or a dedicated field).
- Compat matrix (JSON/TOML) parsed and used to scan scripts/services for risky patterns during preflight.
- UX: set INFO to decisions/actions; demote success chatter to DEBUG in human logs.

## 4) Failure Modes & Guarantees

- Missing sources, untrusted sources, ro/noexec mounts → abort before commit.
- Re-running preflight yields identical plan for same inputs.

## 5) Preflight & Post-Verification

- Preflight runs by default unless `--assume-yes`.
- Post-commit verify: re-run preflight; expect zero planned changes.

## 6) Observability & Audit

- `event=preflight`, `items=N`, `policy_failures=[…]` logged via `audit_event_fields`.

## 7) Security & Policy

- Trust checks for custom `--bin_dir/--unified_binary` sources (use `Worker.verify_owner_for_target`).
- Safety decisions integration (see `PROJECT_PLANS/SAFETY_DECISIONS_AUDIT.md`):
  - Preflight detects `security.capability` and ACLs on targets and emits warnings/remediation hints without mutating by default.
  - Provenance surfaces in plan rows via package ownership and repo presence checks (see `src/system/worker/packages.rs::{query_file_owner, repo_has_package}`).
  - Preflight becomes the default posture unless `--assume-yes`; explicit approval required to commit.

## 8) Migration Plan

1. Implement plan-row builder, renderer, and CLI flag plumbing.
2. Introduce compat matrix and scanner; integrate into preflight path.
3. Adjust human log levels in `logging/init.rs`.

## 9) Testing Strategy

- Unit: plan renderer, compat matcher.
- Integration: preflight before/after → no-op after commit.

## 10) Acceptance Criteria

- `--dry-run`/`--preflight` prints complete plan and never mutates.
- Compat detectors run and block/warn according to matrix.
- Preflight is the default unless `--assume-yes`; commit path requires explicit confirmation.
- Plan rows include `{current_kind,dest?, planned_kind,dest?, policy_ok, provenance}` and capability/ACL findings.
- No duplicate implementations are introduced: plan building reuses `experiments/*` discovery; logging uses `audit_event_fields`; PATH lookups go through `Worker.which()`.

## 11) References

- `src/experiments/util.rs::{create_symlinks, log_applets_summary}`
- `src/cli/{parser.rs, handler.rs}`
- `src/logging/init.rs`
- `PROJECT_PLANS/2_PREFLIGHT_...md`, `8_COMPAT_MATRIX_...md`, `9_UX_CLI_VERBOSITY_...md`
