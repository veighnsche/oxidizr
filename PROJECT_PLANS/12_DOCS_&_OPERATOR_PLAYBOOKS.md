# DOCS & OPERATOR PLAYBOOKS

## 1) Scope (What this project changes)

- Explicit boundaries and non-goals.
  - Produce operator-facing documentation: recovery playbook, files touched, audit artifacts, exit code table.
  - Non-goal: change code behavior.
- Source files/modules likely touched (paths + key symbols).
  - Documentation only; source references for evidence.
  - Evidence: `src/error.rs::Error::exit_code`, `src/state/mod.rs`, `src/symlink/ops.rs`, `src/system/hook.rs`.
- User-visible behavior changes.
  - Clear docs for recovery and audit verification.

## 2) Rationale & Safety Objectives

- Why.
  - Human-auditable guidance reduces operational risk.
- Safety invariants.
  - Accurate, repeatable procedures.

## 3) Architecture & Design

- High-level approach.
  - Write standalone docs describing rollback commands, PATH escape, audit artifacts, and exit codes.

## 4) Failure Modes & Guarantees

- Docs must be updated when interfaces change; tracked in PR checklist.

## 5) Preflight & Post-Change Verification

- Review docs against code symbols; integrate doc checks.

## 6) Observability & Audit

- Include examples of `audit-<op_id>.jsonl` and `.sig` verification.

## 7) Security & Policy

- Include supply-chain policy and AUR opt-in guidance.

## 8) Migration Plan

- Publish docs alongside releases; link from README.

## 9) Testing Strategy

- Doc lint; spell-check; CI link checks.

## 10) Acceptance Criteria (Must be true to ship)

- Docs list recovery steps, touched files, audit verification, exit codes.

## 11) Work Breakdown & Review Checklist

- Draft → review against symbols → publish.

## 12) References (Repo evidence only)

- `src/error.rs::Error::exit_code`
- `src/state/mod.rs::{save_state, write_state_report}`
- `src/symlink/ops.rs::{restore_file, atomic_symlink_swap}`
- `src/system/hook.rs::{hook_body, install_pacman_hook}`
- TODO_LIST_V2.md items: "Recovery playbook and side-effects", "Exit code mapping"
