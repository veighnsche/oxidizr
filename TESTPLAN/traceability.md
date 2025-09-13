# Traceability Matrix (Function × Axis × Level → Scenario IDs)

This matrix maps each function’s axes and levels (from `api_option_inventory.md`) to at least one selected scenario ID (from `test_selection_matrix.md`).

Legend: If a level is intentionally unmapped, justification is provided. Otherwise, every level has at least one scenario.

## plan()

- link_count
  - min:0 → E2E-PLAN-001
  - one → E2E-PLAN-006
  - many:10 → E2E-PLAN-002
  - huge:1000 → E2E-PLAN-004
- restore_count
  - min:0 → E2E-PLAN-001
  - one → E2E-PLAN-002
  - many:10 → E2E-PLAN-005
- duplicate_targets
  - false → E2E-PLAN-002
  - true → E2E-PLAN-003
- input_order
  - sorted → E2E-PLAN-001, E2E-PLAN-006
  - reverse → E2E-PLAN-003
  - random(seed=42) → E2E-PLAN-002

## preflight()

- rescue.require
  - false → E2E-PREFLIGHT-004
  - true → E2E-PREFLIGHT-001
- rescue.exec_check
  - false → E2E-PREFLIGHT-010
  - true → E2E-PREFLIGHT-005
- rescue.min_count
  - min:0 → E2E-PREFLIGHT-004
  - nominal:1 → E2E-PREFLIGHT-001
  - huge:100 → E2E-PREFLIGHT-005
- risks.ownership_strict
  - false → E2E-PREFLIGHT-004
  - true → E2E-PREFLIGHT-002
- durability.preservation
  - Off → E2E-PREFLIGHT-004
  - RequireBasic → E2E-PREFLIGHT-003
- apply.extra_mount_checks.count
  - min:0 → E2E-PREFLIGHT-004
  - one → E2E-PREFLIGHT-008
  - many:5 → E2E-PREFLIGHT-006
- backup.tag
  - empty → E2E-PREFLIGHT-009
  - coreutils → E2E-PREFLIGHT-011
  - long:256 → E2E-PREFLIGHT-007

## apply()

- mode
  - DryRun → E2E-APPLY-001, E2E-APPLY-018
  - Commit → E2E-APPLY-002
- governance.locking
  - Optional → E2E-APPLY-001
  - Required → E2E-APPLY-002, E2E-APPLY-003, E2E-APPLY-015
- lock_manager
  - present → E2E-APPLY-002, E2E-APPLY-015
  - absent → E2E-APPLY-003
- lock_timeout_ms
  - min:0 → E2E-APPLY-003
  - nominal:100 → E2E-APPLY-015
  - max:10000 → E2E-APPLY-010
- apply.override_preflight
  - false → E2E-APPLY-006
  - true → E2E-APPLY-001
- apply.exdev
  - Fail → E2E-APPLY-019
  - DegradedFallback → E2E-APPLY-005
- apply.best_effort_restore
  - false → E2E-APPLY-002
  - true → E2E-APPLY-008
- apply.capture_restore_snapshot
  - false → E2E-APPLY-016
  - true → E2E-APPLY-009
- governance.smoke
  - Off → E2E-APPLY-002
  - Require:auto_rollback=false → E2E-APPLY-004
  - Require:auto_rollback=true → E2E-APPLY-012, E2E-APPLY-011
- smoke_runner
  - present → E2E-APPLY-012, E2E-APPLY-011, E2E-APPLY-018
  - absent → E2E-APPLY-004
- smoke_outcome
  - ok → E2E-APPLY-012
  - fail → E2E-APPLY-011
- risks.ownership_strict
  - false → E2E-APPLY-002
  - true → E2E-APPLY-006, E2E-APPLY-017
- ownership_oracle
  - present → E2E-APPLY-017
  - absent → E2E-APPLY-006
- attestor
  - present → E2E-APPLY-007, E2E-APPLY-013
  - absent → E2E-APPLY-002
- attestation_outcome
  - ok → E2E-APPLY-007
  - error → E2E-APPLY-013
- durability.sidecar_integrity
  - true → E2E-APPLY-002
  - false → E2E-APPLY-020
- durability.backup_durability
  - true → E2E-APPLY-002
  - false → E2E-APPLY-021

## plan_rollback_of()

- apply.capture_restore_snapshot
  - false → E2E-APPLY-016 → E2E-ROLLBACK-001/003
  - true → E2E-APPLY-009 → E2E-ROLLBACK-002/003
- apply_report.executed_shape
  - only_symlink → E2E-ROLLBACK-001
  - only_restore → E2E-ROLLBACK-002
  - mixed → E2E-ROLLBACK-003

## prune_backups()

- backup.tag
  - default → E2E-PRUNE-001
  - coreutils → E2E-PRUNE-004
  - long:256 → E2E-PRUNE-003
- retention_count_limit
  - none → E2E-PRUNE-005
  - min:0 → E2E-PRUNE-008
  - one:1 → E2E-PRUNE-006
  - many:5 → E2E-PRUNE-007
- retention_age_limit
  - none → E2E-PRUNE-005
  - min:1s → E2E-PRUNE-010
  - nominal:1d → E2E-PRUNE-002
  - max:365d → E2E-PRUNE-009

## SafePath::from_rooted()

- root_is_absolute
  - true → E2E-SAFEPATH-002
  - invalid:false → E2E-SAFEPATH-006
- candidate_form
  - relative_normal → E2E-SAFEPATH-004
  - absolute_inside_root → E2E-SAFEPATH-002
  - absolute_outside_root:invalid → E2E-SAFEPATH-003
  - dotdot:invalid → E2E-SAFEPATH-001
  - curdir_components → E2E-SAFEPATH-005
  - unsupported_component:invalid → E2E-SAFEPATH-007
- path_length
  - empty → E2E-SAFEPATH-008
  - short:3segs → E2E-SAFEPATH-010
  - long:255bytes → E2E-SAFEPATH-011
  - huge:4096bytes → E2E-SAFEPATH-012
- unicode
  - false → E2E-SAFEPATH-002
  - true → E2E-SAFEPATH-009
