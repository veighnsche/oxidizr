# SPEC Changes — Consolidated (AI 1–4)

Generated: 2025-09-12
Authors: AI 1, AI 2, AI 3, AI 4 (consolidated)
Inputs: `DOCS/proposals/SPEC_CHANGES_AI{1,2,3,4}.md`, `SPEC/SPEC.md`, `SPEC/audit_event.v2.schema.json`, code in `src/**`

## Overview

This document consolidates the strongest SPEC changes proposed by AI 1–4. Each change includes:

- What changes (normative requirements)
- Why (rationale)
- Affected SPEC sections/files
- New/changed schemas or policy fields (if any)

This consolidation intentionally does not prescribe back-compat/deprecation windows. Changes may be breaking where noted.

---

## 1) SafePath mandatory for all mutating public APIs (breaking)

- What (normative)
  - All mutating public APIs MUST accept `SafePath` (no raw `&Path` variants).
  - TOCTOU-safe syscall sequence is required (open parent O_DIRECTORY|O_NOFOLLOW → *at → renameat → fsync(parent)).
- Why
  - Enforces traversal and TOCTOU safety at the type boundary; eliminates entire class of path footguns.
- Affected
  - SPEC §3 Public Interfaces; SPEC §2.6 (Backup/Restore sequence)
- Notes
  - CLI/SDK guidance MUST show SafePath construction and error handling.

---

## 2) SUID/SGID preflight gate (default deny with policy override)

- What (normative)
  - Preflight MUST detect SUID/SGID bits (via stat()) on target files.
  - When SUID/SGID set and `policy.allow_suid_sgid_mutation=false`, preflight SHALL STOP.
  - Preflight rows SHALL include `suid_sgid_risk=true|false`.
- Why
  - Prevents accidental privilege escalation during system tool mutations.
- Affected
  - SPEC §4 Preflight; SPEC §2.5 Safety and Conservatism (policy defaults)
- Schema/Policy
  - Policy: `allow_suid_sgid_mutation: bool` (default false in production presets).

---

## 3) Backup durability requirements

- What (normative)
  - Backup payloads: `sync_all()` before close.
  - Sidecars: `sync_all()` after write.
  - Parent directory: `fsync_parent_dir()` after all artifacts are created/renamed.
  - Directory operations SHALL use capability handles (`open_dir_nofollow` + *at syscalls).
- Why
  - Ensures backup survival across crashes; strengthens recovery guarantees.
- Affected
  - SPEC §2.6.4 Backup Durability; syscall sequence amplify.
- Schema/Policy
  - Policy: `require_backup_durability: bool` (default true) influences enforcement.
  - Facts: MAY include `backup_durable=true|false` in apply facts.

---

## 4) Sidecar integrity with payload hash and restore verification

- What (normative)
  - Sidecar schema v2 SHALL include `payload_hash` (e.g., SHA-256 of payload).
  - On backup: compute/store the hash; on restore: verify hash before proceeding.
  - If verification fails and policy requires integrity, restore SHALL fail with `E_BACKUP_TAMPERED`.
  - Restore facts SHALL include `sidecar_integrity_verified=true|false`.
- Why
  - Detects tampering/corruption; raises confidence in restores.
- Affected
  - SPEC §2.6.5 Sidecar Integrity and Verification; §13 Facts Schema for new field.
- Schema/Policy
  - Sidecar v2: `payload_hash: string` (optional for v1 coexistence but normative for new backups).
  - Policy: `require_sidecar_integrity: bool` (default true in production presets).

---

## 5) Preservation tiers and sidecar v2 schema

- What (normative)
  - Introduce `Policy.preservation_tier`: Basic (mode), Extended (mode+uid+gid+mtime), Full (Extended+xattrs+ACLs/caps).
  - Sidecar v2 SHALL include optional `uid`, `gid`, `mtime_sec`, `mtime_nsec`, and `xattrs` consistent with tier and capabilities.
  - Restore SHALL apply preserved metadata when capabilities permit.
- Why
  - Metadata fidelity for migrations/backups; predictable behavior across environments.
- Affected
  - SPEC §2.6.3 Preservation Fidelity; §4 Preflight; §13 Schema versioning.
- Schema/Policy
  - Policy: `preservation_tier: Basic|Extended|Full`.

---

## 6) Immutable bit detection requirements (reliable preflight)

- What (normative)
  - Preflight immutable detection precedence: ioctl(FS_IOC_GETFLAGS) → `lsattr` fallback → `unknown`.
  - When `immutable_check=unknown`, preflight SHALL STOP unless `Policy.allow_unreliable_immutable_check=true`.
  - Preflight rows SHALL include `immutable_detection_method`.
- Why
  - Works in minimal environments while remaining conservative by default.
- Affected
  - SPEC §4.2 Immutable File Detection; Policy section.
- Schema/Policy
  - Policy: `allow_unreliable_immutable_check: bool` (default false).

---

## 7) Retention policy knobs and invariants

- What (normative)
  - Policy SHALL support `retention_count_limit: Option<u32>` and `retention_age_limit: Option<Duration>`.
  - Pruning MUST NOT delete the last remaining valid backup; deletes MUST be atomic across payload+sidecar.
- Why
  - Prevent unbounded disk usage; keep reliable rollback path.
- Affected
  - SPEC §2.7 Backup Retention and Pruning.

---

## 8) Facts schema — `summary_error_ids` array in summaries

- What (normative)
  - `preflight.summary`, `apply.result.summary`, and rollback summaries SHALL support an optional `summary_error_ids: [string]` ordered from most specific to most general, alongside existing `error_id`.
- Why
  - Improves observability and policy by surfacing all relevant error classes.
- Affected
  - SPEC §13 Audit Event Schema; `SPEC/audit_event.v2.schema.json`.

---

## 9) Error taxonomy: E_OWNERSHIP co-emission and chain semantics

- What (normative)
  - When ownership-related checks fail, emit `E_OWNERSHIP` in addition to specific error IDs; include all IDs in `summary_error_ids`.
- Why
  - Enables coarse-grained routing/alerting while preserving specificity.
- Affected
  - SPEC §6 Error Taxonomy; §13 Schema semantics.

---

## 10) Facts schema — `perf` object in summaries (apply/rollback)

- What (normative)
  - Summaries MAY include `perf` object with timing aggregates such as `total_fsync_ms`, `total_hash_ms`, `total_backup_ms`, `total_restore_ms`.
- Why
  - Enables performance visibility and regression detection.
- Affected
  - SPEC §13 Audit Event Schema; `SPEC/audit_event.schema.json`.

---

## 11) Facts schema — `lock_backend` in apply facts

- What (normative)
  - `apply.attempt` and `apply.result` facts SHALL include `lock_backend` identifying the lock implementation (e.g., "file", "redis", or "none").
- Why
  - Essential for diagnostics, contention analysis, and fleet observability.
- Affected
  - SPEC §13 Apply Facts.

---

## 12) SPEC: CLI integration guidance for SafePath

- What (normative)
  - SPEC SHALL include a concise section describing SafePath-based path validation for CLI integrations, with examples.
  - Documentation SHALL avoid referencing non-existent functions; proposed or future work MUST be clearly labeled.
- Why
  - Prevents documentation drift and unsafe example patterns; improves DX.
- Affected
  - SPEC §3.2 CLI Integration and SafePath Guidance.

---

## Schema delta (summary)

Proposed additive changes to `SPEC/audit_event.v2.schema.json`:

- Add `summary_error_ids: array[string]` (summaries)
- Add optional `perf: object { total_fsync_ms, total_hash_ms, total_backup_ms, total_restore_ms }` (apply/rollback summaries)
- Add `lock_backend: string` (apply.attempt/apply.result)
- Add `sidecar_integrity_verified: boolean` (restore-related facts)

Sidecar schema v2: add `payload_hash`, and (for preservation tiers) optional `uid`, `gid`, `mtime_sec`, `mtime_nsec`, `xattrs`.

---

## Policy delta (summary)

- `allow_suid_sgid_mutation: bool` (default false in production presets)
- `require_backup_durability: bool` (default true)
- `require_sidecar_integrity: bool` (default true in production presets)
- `preservation_tier: Basic|Extended|Full`
- `allow_unreliable_immutable_check: bool` (default false)
- `retention_count_limit: Option<u32>`
- `retention_age_limit: Option<Duration>`

---

## Rationale at a glance

- SafePath-only at boundaries removes entire classes of traversal/TOCTOU issues.
- SUID/SGID and immutable gates set secure defaults, with explicit opt-outs.
- Durability and integrity requirements make recovery reliable in real-world failures.
- Retention protects disk while preserving a guaranteed rollback path.
- Observability (`summary_error_ids`, `perf`, `lock_backend`) enables analytics, alerting, and performance SLOs.
- Preservation tiers align fidelity with capabilities and use cases.
- Clear CLI guidance prevents documentation drift and unsafe examples.
