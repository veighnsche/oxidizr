# SPEC Change Proposals — AI 3
Generated: 2025-09-12 16:33:01+02:00
Author: AI 3
Inputs: [RETENTION_STRATEGY.md], [PERFORMANCE_PLAN.md], [SPEC §13], [src/policy/config.rs], [src/fs/backup.rs]

## Proposal 1: Retention Policy Knobs and Invariants

- **Motivation (why):** The system currently accumulates backups indefinitely, leading to unbounded disk usage. The analysis in `RETENTION_STRATEGY.md` confirmed the absence of any pruning mechanism, which is a critical operational feature for production environments.
- **Current spec:** SPEC does not currently define any retention policies or pruning behavior.
- **Proposed change (normative):
  - **Add:** New SPEC §2.7 "Backup Retention and Pruning"

    ```text
    The engine SHALL support backup retention policies configurable via `Policy`.
    The following policy knobs SHALL be available:
    - `retention_count_limit: Option<u32>`: The maximum number of backups to retain per target.
    - `retention_age_limit: Option<Duration>`: The maximum age of backups to retain per target.

    Pruning operations MUST NOT delete the last remaining valid backup for a given target, to ensure at least one rollback path is always available.
    Pruning SHALL be performed atomically, ensuring both the backup payload and its corresponding sidecar are removed together.
    ```
  - **Affected sections:** New SPEC §2.7, `policy/config.rs`.

- **Compatibility & migration:**
  - **Backward compatibility:** Yes. The feature is opt-in, with retention limits defaulting to `None` (infinite retention), matching current behavior.
  - **Migration plan:** The new policy knobs can be added in the next minor release. A `prune_backups` feature will implement the logic.

- **Security & privacy:**
  - **Impact:** Neutral. This feature manages disk space and does not introduce new security concerns.

- **Acceptance criteria:**
  - `Policy` struct in `src/policy/config.rs` contains the new optional retention fields.
  - A `prune_backups` function respects the policy knobs.
  - Tests verify that pruning never deletes the last valid backup.
  - Tests verify that both sidecar and payload are removed.

- **Evidence:**
  - **Analysis:** `RETENTION_STRATEGY.md` (Round 1 analysis confirming no retention exists).
  - **Code:** `src/fs/backup.rs::backup_path_with_tag()` (shows timestamped name creation), `src/fs/backup.rs::find_latest_backup_and_sidecar()` (shows discovery mechanism).

## Proposal 2: Performance Telemetry in Summaries

- **Motivation (why):** The analysis in `PERFORMANCE_PLAN.md` identified I/O hotspots like hashing and `fsync`, but there is no structured way to report these timings in facts. This limits performance diagnostics and the ability to monitor latency SLAs.
- **Current spec:** SPEC §13 (Facts Schema) does not define a container for performance metrics.
- **Proposed change (normative):
  - **Add:** In `apply.result` and `rollback.result` summaries, define an optional field `perf: object`.

    ```json
    "perf": {
      "type": "object",
      "properties": {
        "total_fsync_ms": { "type": "integer" },
        "total_hash_ms": { "type": "integer" },
        "total_backup_ms": { "type": "integer" },
        "total_restore_ms": { "type": "integer" }
      }
    }
    ```
  - **Affected sections:** SPEC §13, `cargo/switchyard/SPEC/audit_event.schema.json`.

- **Compatibility & migration:**
  - **Backward compatibility:** Yes. The `perf` object is optional and additive.
  - **Migration plan:** The field can be added to the schema and implemented in emitters in a single release.

- **Security & privacy:**
  - **Impact:** Neutral. The timings are not sensitive data.

- **Acceptance criteria:**
  - `audit_event.schema.json` is updated with the `perf` object definition.
  - Emitters in `src/api/apply/mod.rs` aggregate and populate these timings.
  - Integration tests verify that the `perf` object is present in facts output and contains plausible timings.

- **Evidence:**
  - **Analysis:** `PERFORMANCE_PLAN.md` (Round 1 analysis identifying hotspots).
  - **Code:** `src/fs/meta.rs::sha256_hex_of()`, `src/fs/atomic.rs`, `src/fs/backup.rs` (locations where expensive I/O occurs).

---

Proposals authored by AI 3 on 2025-09-12 16:33:01+02:00
