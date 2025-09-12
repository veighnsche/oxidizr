# SPEC Change Proposals — AI 2

Generated: 2025-09-12 16:27 +02:00  
Author: AI 2  
Inputs: [PRESERVATION_FIDELITY.md], [PREFLIGHT_MODULE_CONCERNS.md], [POLICY_PRESETS_RATIONALE.md], [LOCKING_STRATEGY.md], [SECURITY_REVIEW.md], [RELEASE_AND_CHANGELOG_POLICY.md], [SPEC §2.6, §4, §13]

## Proposal 1: Preservation Tiers and Sidecar v2 Schema

- **Motivation (why):** Current mode-only preservation limits tool applicability in metadata-sensitive environments (backups, system migrations). S2 Priority 3 finding from PRESERVATION_FIDELITY.md Round 3 assessment. Users expect comprehensive metadata preservation for system integrity.

- **Current spec:** SPEC §2.6 Backup and Restore mentions mode preservation but lacks extended metadata specification.

- **Proposed change (normative):**
  - Add/Modify: SPEC §2.6.3 "Preservation Fidelity"

    ```text
    The engine SHALL support three preservation tiers via Policy.preservation_tier:
    - Basic: mode only (current behavior)
    - Extended: mode + uid + gid + mtime (when capabilities permit)
    - Full: Extended + xattrs + ACLs (when capabilities permit)
    
    Sidecar schema v2 SHALL include optional fields: uid, gid, mtime_sec, mtime_nsec, xattrs.
    Restore operations SHALL apply preserved metadata when tier >= Extended and capabilities exist.
    Capability detection SHALL be performed during preflight via detect_preservation_capabilities().
    ```

  - Affected sections: SPEC §2.6, §4 (preflight rows), §13 (schema versioning)

- **Compatibility & migration:**
  - Backward compatibility: Yes, via optional sidecar fields and tier defaults to Basic
  - Migration plan: v2 sidecars coexist with v1; Extended/Full tiers opt-in via policy

- **Security & privacy:**
  - Impact: Enhanced system integrity through complete metadata preservation; no privacy concerns

- **Acceptance criteria:**
  - Sidecar v2 schema validates with extended fields
  - Policy.preservation_tier enum implemented with three levels
  - Preflight rows include preservation_tier and preservation_applied fields
  - Unit tests verify metadata round-trip for each tier
  - Graceful degradation when capabilities unavailable

- **Evidence:**
  - Code: `src/fs/backup.rs::create_snapshot()` lines 194–205, `src/fs/restore.rs::restore_file()` lines 129–137
  - Analysis: `PRESERVATION_FIDELITY.md` Round 3 assessment lines 144–151

## Proposal 2: Immutable Bit Detection Requirements

- **Motivation (why):** lsattr dependency fails in minimal environments, allowing operations on immutable files to proceed undetected. S2 Priority 3 finding from PREFLIGHT_MODULE_CONCERNS.md. Robust preflight checks require reliable detection methods.

- **Current spec:** SPEC §4 Preflight mentions immutable checks but doesn't specify detection method requirements.

- **Proposed change (normative):**
  - Add: SPEC §4.2 "Immutable File Detection"

    ```text
    Preflight checks SHALL detect immutable files using the following precedence:
    1. ioctl(FS_IOC_GETFLAGS) when available
    2. lsattr fallback when ioctl unavailable
    3. Report immutable_check=unknown when both fail
    
    When immutable_check=unknown, preflight SHALL STOP unless Policy.allow_unreliable_immutable_check=true.
    Preflight rows SHALL include immutable_detection_method field: "ioctl"|"lsattr"|"unknown".
    ```

  - Affected sections: SPEC §4, Policy configuration

- **Compatibility & migration:**
  - Backward compatibility: Yes, lsattr remains as fallback
  - Migration plan: ioctl detection added as primary method in next release

- **Security & privacy:**
  - Impact: Prevents undetected operations on immutable files; enhanced reliability

- **Acceptance criteria:**
  - ioctl detection implemented with FS_IOC_GETFLAGS
  - Fallback chain: ioctl → lsattr → unknown
  - Policy flag allow_unreliable_immutable_check controls unknown handling
  - Preflight facts include detection method and reliability status
  - Unit tests cover all detection paths and policy interactions

- **Evidence:**
  - Code: `src/preflight/checks.rs::check_immutable()` lines 20–41
  - Analysis: `PREFLIGHT_MODULE_CONCERNS.md` Round 3 assessment lines 153–160

## Proposal 3: Backup Durability Requirements

- **Motivation (why):** Missing fsync operations risk data loss during crashes, undermining core recovery features. S3 Priority 2 finding from PRESERVATION_FIDELITY.md. Durability guarantees essential for backup reliability.

- **Current spec:** SPEC §2.6 doesn't specify durability requirements for backup operations.

- **Proposed change (normative):**
  - Add: SPEC §2.6.4 "Backup Durability"

    ```text
    Backup creation SHALL ensure durability via:
    1. File payloads: sync_all() before close
    2. Symlink payloads: symlinkat() with directory handle
    3. Sidecars: sync_all() after write
    4. Parent directories: fsync_parent_dir() after all artifacts created
    
    Directory operations SHALL use open_dir_nofollow() + *at syscalls for TOCTOU safety.
    Policy.require_backup_durability (default true) controls durability enforcement.
    Apply facts SHALL include backup_durable=true|false field.
    ```

  - Affected sections: SPEC §2.6, §13 (facts schema)

- **Compatibility & migration:**
  - Backward compatibility: Yes, durability is additive safety enhancement
  - Migration plan: Immediate implementation with policy control

- **Security & privacy:**
  - Impact: Enhanced reliability; prevents backup loss during system failures

- **Acceptance criteria:**
  - All backup operations use directory handles and *at syscalls
  - fsync_parent_dir() called after backup creation
  - Policy flag controls durability requirements
  - Apply facts include durability status
  - Integration tests verify backup survival across simulated crashes

- **Evidence:**
  - Code: `src/fs/backup.rs::write_sidecar()` lines 262–270, `src/fs/backup.rs::create_snapshot()` lines 137–151
  - Analysis: `PRESERVATION_FIDELITY.md` Round 3 assessment lines 153–160

## Proposal 4: Lock Backend Telemetry Requirements

- **Motivation (why):** Missing lock_backend field limits ops teams' diagnostic capabilities. S4 Priority 1 finding from LOCKING_STRATEGY.md. Fleet-wide lock analysis requires backend identification.

- **Current spec:** SPEC §13 Facts Schema doesn't include lock backend identification.

- **Proposed change (normative):**
  - Add/Modify: SPEC §13.3 "Apply Facts"

    ```text
    apply.attempt and apply.result facts SHALL include:
    - lock_backend: string identifying lock implementation ("file", "redis", etc.)
    - lock_wait_ms: duration spent acquiring lock (existing)
    
    When no LockManager configured, lock_backend SHALL be "none".
    ```

  - Affected sections: SPEC §13

- **Compatibility & migration:**
  - Backward compatibility: Yes, additive field
  - Migration plan: Immediate addition to facts schema

- **Security & privacy:**
  - Impact: Improved observability; no security implications

- **Acceptance criteria:**
  - lock_backend field present in apply facts
  - Field correctly identifies backend type
  - "none" value when no lock manager configured
  - Facts schema validation includes new field

- **Evidence:**
  - Code: `src/api/apply/mod.rs` lines 355–357
  - Analysis: `LOCKING_STRATEGY.md` Round 3 assessment lines 128–135

## Proposal 5: Deprecation and CHANGELOG Requirements

- **Motivation (why):** Missing deprecation warnings risk integration breakage during upgrades. S2 Priority 3 finding from RELEASE_AND_CHANGELOG_POLICY.md. Smooth API transitions require clear communication.

- **Current spec:** No specification for deprecation handling or changelog requirements.

- **Proposed change (normative):**
  - Add: New SPEC §14 "API Lifecycle and Deprecation"

    ```text
    Deprecated APIs SHALL include #[deprecated] attributes with:
    - Clear migration guidance
    - Target removal version
    - Alternative API recommendations
    
    CHANGELOG.md SHALL be maintained at crate level with sections:
    - Added, Changed, Deprecated, Removed, Fixed, Security
    - Public API changes SHALL require CHANGELOG updates
    - CI SHALL enforce changelog updates when cargo public-api detects diffs
    ```

  - Affected sections: New SPEC §14

- **Compatibility & migration:**
  - Backward compatibility: Yes, establishes process for future changes
  - Migration plan: Apply to current shims and future deprecations

- **Security & privacy:**
  - Impact: Reduced integration risk; improved upgrade safety

- **Acceptance criteria:**
  - Deprecation attributes on legacy shims
  - CHANGELOG.md exists with proper sections
  - CI gate enforces changelog updates on API changes
  - Documentation includes deprecation timeline examples

- **Evidence:**
  - Code: `src/adapters/mod.rs` lines 6–9, `src/fs/mod.rs` lines 9–15
  - Analysis: `RELEASE_AND_CHANGELOG_POLICY.md` Round 3 assessment lines 84–91

---

Proposals authored by AI 2 on 2025-09-12 16:27 +02:00
