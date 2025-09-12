# SPEC Change Proposals — AI 4
Generated: 2025-09-12 16:34 CET
Author: AI 4
Inputs: DOCS/analysis/EDGE_CASES_AND_BEHAVIOR.md, DOCS/analysis/CORE_FEATURES_FOR_EDGE_CASES.md, DOCS/analysis/CLI_INTEGRATION_GUIDE.md, SPEC §2.5, §2.6, §3, §4, §13

## Proposal 1: SUID/SGID Preflight Gate with Default Deny and Override Flag
- Motivation (why): Modifying SUID/SGID binaries without explicit consent poses significant security risks, potentially leading to privilege escalation. Identified as S3 Priority 2 in CORE_FEATURES_FOR_EDGE_CASES.md Round 3 assessment. A preflight gate is essential to prevent accidental modifications.
- Current spec: SPEC §4 Preflight does not mention checks for SUID/SGID bits or related policy controls.
- Proposed change (normative):
  - Add: SPEC §4.3 "SUID/SGID Binary Protection Gate"
    ```text
    Preflight checks SHALL detect SUID/SGID bits on target files using stat() or equivalent syscall.
    If SUID or SGID bits are present, preflight SHALL emit a STOP row with suid_sgid_risk=true.
    Policy.allow_suid_sgid_mutation (default false in production_preset and coreutils_switch_preset) controls whether operations on such binaries are permitted.
    Preflight rows SHALL include suid_sgid_risk=true|false field.
    ```
  - Affected sections: SPEC §4, §2.5 Safety and Conservatism (policy defaults)
- Compatibility & migration:
  - Backward compatibility: Yes, additive check with policy override for existing behavior.
  - Migration plan: Implement in next release with default deny; users can opt-in to allow via policy flag.
- Security & privacy:
  - Impact: Significantly reduces risk of accidental privilege escalation by enforcing explicit consent for SUID/SGID modifications.
- Acceptance criteria:
  - Preflight check for SUID/SGID bits implemented in `src/preflight/checks.rs`.
  - Policy flag `allow_suid_sgid_mutation` added with default `false` in production presets.
  - STOP row emitted with `suid_sgid_risk=true` when detected and policy disallows.
  - Unit and integration tests cover detection and policy override scenarios.
- Evidence:
  - Code: `src/preflight/checks.rs` (current checks lack SUID/SGID detection)
  - Analysis: DOCS/analysis/CORE_FEATURES_FOR_EDGE_CASES.md Round 3 assessment (planned but not yet documented in detail)

## Proposal 2: Sidecar Integrity with Payload Hash and Restore Verification
- Motivation (why): Without integrity checks, backup sidecars can be tampered with, leading to unreliable restores and potential security risks. Identified as S3 Priority 2 in EDGE_CASES_AND_BEHAVIOR.md Round 3 assessment. Integrity verification ensures trust in recovery mechanisms.
- Current spec: SPEC §2.6 Backup and Restore does not specify integrity mechanisms for sidecar data.
- Proposed change (normative):
  - Add: SPEC §2.6.5 "Sidecar Integrity and Verification"
    ```text
    Backup sidecar schema v2 SHALL include a payload_hash field (e.g., SHA-256 of the backup payload).
    During backup creation, the engine SHALL compute and store the hash of the payload in the sidecar.
    During restore, the engine SHALL verify the stored payload hash against the sidecar's payload_hash before proceeding.
    Policy.require_sidecar_integrity (default true in production_preset) controls whether integrity verification is mandatory.
    Restore facts SHALL include sidecar_integrity_verified=true|false field.
    If verification fails and policy requires integrity, restore SHALL fail with E_BACKUP_TAMPERED.
    ```
  - Affected sections: SPEC §2.6, §13 (facts schema)
- Compatibility & migration:
  - Backward compatibility: Yes, via schema versioning (v2 sidecars coexist with v1); policy default ensures enforcement in production.
  - Migration plan: Implement v2 sidecar schema with hash field; existing v1 sidecars remain compatible without verification.
- Security & privacy:
  - Impact: Enhances security by preventing tampered backups from being restored, ensuring recovery integrity.
- Acceptance criteria:
  - Sidecar schema v2 includes `payload_hash` field.
  - Backup creation computes and stores payload hash in `src/fs/backup.rs`.
  - Restore operation verifies hash in `src/fs/restore.rs` and fails with `E_BACKUP_TAMPERED` if policy requires integrity.
  - Policy flag `require_sidecar_integrity` defaults to `true` in production presets.
  - Restore facts include integrity verification status.
  - Tests cover tampering scenarios and policy interactions.
- Evidence:
  - Code: `src/fs/backup.rs::BackupSidecar` (lines 244–252), `src/fs/restore.rs::restore_file()` (lines 96–107)
  - Analysis: DOCS/analysis/EDGE_CASES_AND_BEHAVIOR.md Round 3 assessment (aligned with SECURITY_REVIEW.md findings)

## Proposal 3: Enhanced CLI Integration Guidance in SPEC for SafePath Usage
- Motivation (why): Current CLI integration guidance lacks clarity on `SafePath` usage and references non-existent functions, leading to developer confusion and potential security risks. Identified as S2 Priority 3 in CLI_INTEGRATION_GUIDE.md Round 3 assessment. Clear SPEC guidance ensures safe usage patterns.
- Current spec: SPEC §3 Public Interfaces mentions `SafePath` but lacks detailed guidance for CLI integration.
- Proposed change (normative):
  - Add: SPEC §3.2 "CLI Integration and SafePath Guidance"
    ```text
    CLI integrations SHALL validate all path inputs using SafePath or equivalent traversal protection before passing to library APIs.
    Documentation for CLI tools SHALL include examples of SafePath construction and error handling for invalid paths.
    Documentation SHALL NOT reference non-existent functions; if functionality is planned, it SHALL be marked as 'future work' with a reference to roadmap or issue tracking.
    During the transition period to full SafePath enforcement, documentation SHALL provide interim best practices for secure path handling with raw Path inputs.
    ```
  - Affected sections: SPEC §3
- Compatibility & migration:
  - Backward compatibility: Yes, additive guidance for documentation and integration practices.
  - Migration plan: Update CLI integration documentation in next release to align with this guidance; full `SafePath` enforcement to follow deprecation timeline.
- Security & privacy:
  - Impact: Improves security by guiding developers toward safe path handling practices, reducing traversal risks.
- Acceptance criteria:
  - CLI integration documentation (e.g., `CLI_INTEGRATION_GUIDE.md`) includes `SafePath` examples and error handling.
  - Non-existent functions are removed from documentation or marked as 'future work'.
  - Interim best practices for raw `Path` inputs are documented until full `SafePath` enforcement.
  - SPEC §3.2 is added with the proposed text.
- Evidence:
  - Code: `src/types/safepath.rs` (SafePath definition), `src/fs/paths.rs::is_safe_path()` (validation logic)
  - Analysis: DOCS/analysis/CLI_INTEGRATION_GUIDE.md Round 3 assessment (documentation gaps)

---

Proposals authored by AI 4 on 2025-09-12 16:34 CET
