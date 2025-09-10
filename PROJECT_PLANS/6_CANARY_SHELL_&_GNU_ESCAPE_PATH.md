# CANARY SHELL & GNU ESCAPE PATH

## 1) Scope (What this project changes)

- Explicit boundaries and non-goals.
  - Provide a stable GNU profile path and a diagnostics "canary" shell that uses it.
  - Non-goal: bundling BusyBox or additional toolkits.
- Source files/modules likely touched (paths + key symbols).
  - `src/experiments/coreutils.rs::{enable, disable}` (to keep GNU tree intact)
  - `src/experiments/util.rs::{resolve_usrbin}` (escape path docs)
  - `src/cli/{parser.rs, handler.rs}` (subcommand design only)
- User-visible behavior changes.
  - New documented escape hatch: `export PATH=/usr/lib/oxidizr-arch/profiles/gnu/bin:$PATH`.

## 2) Rationale & Safety Objectives

- Why.
  - Ensure an immediate, low-risk GNU recovery path without expanding trusted surface.
- Safety invariants.
  - No external bundles; recovery is a PATH override.
- Overkill → Lean replacement summary.
  - Replace rescue toolkit bundle with GNU escape path + canary shell.

## 3) Architecture & Design

- High-level approach.
  - Maintain a GNU profile under the profiles layout. Provide a `canary --shell` command that launches a shell with PATH prefixed by GNU profile for diagnosis. No mutation to system state.
- Data model.
  - None.
- Control flow.
  - Resolve GNU profile path → spawn shell with modified env.
- Public interfaces.
  - CLI: `oxidizr-arch canary --shell`.
- Filesystem layout.
  - `/usr/lib/oxidizr-arch/profiles/gnu/bin/*`.

## 4) Failure Modes & Guarantees

- Missing GNU tree → print remediation to reinstall GNU provider.
- Idempotency: launching canary shell has no side effects.
- Concurrency: none.

## 5) Preflight & Post-Change Verification

- Preflight: ensure GNU profile exists and is executable.
- Post: N/A (no mutation).

## 6) Observability & Audit

- Log `event=canary_shell_opened`, `profile=gnu`.

## 7) Security & Policy

- No privilege changes; environment-only.

## 8) Migration Plan

- Document feature; implement after profiles layout is available.

## 9) Testing Strategy

- Unit: path computation.
- Manual: operator can run shell and verify `which ls` resolves to GNU tree.

## 10) Acceptance Criteria (Must be true to ship)

- Canary shell starts with GNU PATH and no side effects.

## 11) Work Breakdown & Review Checklist

- Docs → CLI wiring → audit log.

## 12) References (Repo evidence only)

- `src/experiments/coreutils.rs::{enable, disable}`
- `src/experiments/util.rs::resolve_usrbin`
- `src/cli/{parser.rs, handler.rs}`
- TODO_LIST_V2.md items: "GNU escape path + canary shell (replace BusyBox bundle)", "Profiles layout + single active pointer flip"
