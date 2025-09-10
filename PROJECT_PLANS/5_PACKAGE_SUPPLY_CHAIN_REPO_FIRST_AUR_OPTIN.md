# PACKAGE SUPPLY CHAIN: REPO-FIRST & AUR OPT-IN

## 1) Scope (What this project changes)

- Explicit boundaries and non-goals.
  - Enforce repo-first installs; gate AUR behind explicit `--allow-aur` and `--aur-user`.
  - Sanitize environment for helper calls; record provenance.
- Source files/modules likely touched (paths + key symbols).
  - `src/system/worker/packages.rs::{install_package, ensure_aur_preflight, repo_has_package, check_installed}`
  - `src/system/worker/distro.rs::{extra_repo_available}`
  - `src/cli/{parser.rs, handler.rs}` (flags)
- User-visible behavior changes.
  - Without `--allow-aur`, emit the exact helper command instead of running it.

## 2) Rationale & Safety Objectives

- Why.
  - Reduce trusted surface and align with operator consent.
- Safety invariants.
  - No implicit AUR execution; clear provenance logged.
- Overkill → Lean replacement summary.
  - Replace implicit helper orchestration with explicit opt-in.

## 3) Architecture & Design

- High-level approach.
  - Keep `repo_has_package` probe; when absent, require explicit flags to invoke AUR helpers. Construct minimal env for `Command`/`su -c` (e.g., `LC_ALL=C`, pinned PATH). Always log provenance via `pacman -Qo` and binary `--version`.
- Data model.
  - No persistent schema changes.
- Control flow.
  - Probe → install from repo → else require `--allow-aur` → run helper → verify installed.
- Public interfaces.
  - CLI: `--allow-aur`, `--aur-user`.

## 4) Failure Modes & Guarantees

- Missing helper or denied AUR path → clear error with emitted command.
- Lock timeouts → maintain simple bounded wait (`wait_for_pacman_lock_clear`).

## 5) Preflight & Post-Change Verification

- Preflight: verify `pacman`, `pacman-conf`, helper presence when needed.
- Post: verify installation and record package owner.

## 6) Observability & Audit

- Fields: `install_source=repo|aur`, `cmd`, `rc`, `owner_pkg`.

## 7) Security & Policy

- Environment sanitization for helper calls.
- Enforce repo-first with explicit user consent for AUR.

## 8) Migration Plan

- Start with warnings on implicit AUR; switch to hard requirement.

## 9) Testing Strategy

- Matrix: repo present/absent; with/without `--allow-aur`; helper present/absent.

## 10) Acceptance Criteria (Must be true to ship)

- No AUR action without explicit flags; provenance logged.

## 11) Work Breakdown & Review Checklist

- CLI flags → gating checks → env sanitization → tests.

## 12) References (Repo evidence only)

- `src/system/worker/packages.rs::{install_package, ensure_aur_preflight, repo_has_package, wait_for_pacman_lock_clear}`
- `src/system/worker/distro.rs::extra_repo_available`
- `src/cli/{parser.rs, handler.rs}`
- TODO_LIST_V2.md items: "AUR opt-in policy (repo-first)", "Sanitize environment for external commands (AUR helpers, pacman)", "Supply chain: signature verification and SBOM fragments", "Pacman lock handling ergonomics"
