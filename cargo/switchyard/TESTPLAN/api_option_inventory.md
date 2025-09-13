# API Option Inventory (Switchyard Library)

Canonical inventory of public API entry points, their axes (options/flags/modes/variants), levels and boundaries, with risk classifications and constraints. This document is the source for combinatorial modeling and selection.

Terminology:

- Axis: Named option that can vary and affects behavior or contracts.
- Levels: Allowed values for an axis (include boundary markers).
- Risk: High/Medium/Low likelihood×impact for interaction defects or data loss.
- Domain: boolean | enum | range | collection | string.

## Global construction axes (apply to multiple functions)

These are set via `ApiBuilder` (`src/api/builder.rs`) and `Switchyard::with_*` methods (`src/api/mod.rs`).

| Axis | Domain | Levels (+boundaries) | Risk | Notes/Constraints |
|---|---|---|---|---|
| lock_manager | boolean | present, absent | High | If `governance.locking=Required` and `ApplyMode=Commit` and `lock_manager=absent` → expect `E_LOCKING` (negative). |
| ownership_oracle | boolean | present, absent | Medium | Needed to populate provenance and enforce ownership when `ownership_strict=true`. |
| attestor | boolean | present, absent | Low | Only affects attestation fields on successful Commit apply. |
| smoke_runner | boolean | present, absent | High | Required when `SmokePolicy=Require{..}`; else `E_SMOKE` in Commit. |
| lock_timeout_ms | range | <min>=0, nominal=100, <max>=10000 | Medium | When `Required` locking, timeout=0 can trigger immediate failure in negative test. |

## Function: Switchyard::plan(&self, PlanInput) -> Plan

Source: `src/api/plan.rs`, `src/types/plan.rs`

| Axis | Domain | Levels (+boundaries) | Risk | Notes/Constraints |
|---|---|---|---|---|
| plan_input.link_count | collection size | <min>=0, one, many(10), <huge>=1000 | Low | Sorting must be stable regardless of count. |
| plan_input.restore_count | collection size | <min>=0, one, many(10) | Low | Same as above. |
| plan_input.duplicate_targets | boolean | false, true | Medium | Duplicate targets are allowed; plan is not deduped. |
| plan_input.input_order | enum | sorted, reverse, random(seed=42) | Low | Plan sorts by (kind,target.rel) deterministically. |

## Function: Switchyard::preflight(&self, &Plan) -> Result<PreflightReport, ApiError>

Source: `src/api/preflight/`

| Axis | Domain | Levels (+boundaries) | Risk | Notes/Constraints |
|---|---|---|---|---|
| policy.rescue.require | boolean | false, true | High | If `true` and tools unavailable → STOP; negative path expected. |
| policy.rescue.exec_check | boolean | false, true | Medium | Adds runtime verification cost; affects stops when require=true. |
| policy.rescue.min_count | range | <min>=0, nominal=1, <huge>=100 | Medium | With `require=true`, lack of min tools → STOP. |
| policy.risks.ownership_strict | boolean | false, true | High | If `true` and `ownership_oracle=absent` or returns error → STOP. |
| policy.durability.preservation | enum | Off, RequireBasic | High | If `RequireBasic` and FS unsupported → STOP. |
| policy.apply.extra_mount_checks | collection size | <min>=0, one, many(5) | Medium | May cause extra STOPs depending on env mounts. |
| policy.backup.tag | string | <empty>, "coreutils", <long>=256 chars | Low | Used for annotations and prune scoping. |

Constraints:

- IF ownership_strict=true THEN ownership_oracle SHOULD be present (soft constraint; still allowed, but expect STOP).
- IF policy.rescue.require=true AND min_count>0 AND rescue tools missing (env) THEN expect STOP.

## Function: Switchyard::apply(&self, &Plan, ApplyMode) -> Result<ApplyReport, ApiError>
Source: `src/api/apply/`

| Axis | Domain | Levels (+boundaries) | Risk | Notes/Constraints |
|---|---|---|---|---|
| mode | enum | DryRun, Commit | High | DryRun uses `TS_ZERO` timestamps; Commit performs real changes. |
| policy.governance.locking | enum | Optional, Required | High | Required implies lock needed in Commit. |
| lock_manager (global) | boolean | present, absent | High | See constraint with locking Required. |
| lock_timeout_ms (global) | range | <min>=0, nominal=100, <max>=10000 | Medium | 0 combined with Required can simulate immediate timeout. |
| policy.apply.override_preflight | boolean | false, true | High | When false, apply stops if preflight would STOP. |
| policy.apply.exdev | enum | Fail, DegradedFallback | High | Cross-FS swaps fail vs degraded path; env determines EXDEV. |
| policy.apply.best_effort_restore | boolean | false, true | High | If true, missing backup is non-fatal. |
| policy.apply.capture_restore_snapshot | boolean | false, true | Medium | Enables rollback inversion of restore actions. |
| policy.governance.smoke | enum | Off, Require{auto_rollback=false}, Require{auto_rollback=true} | High | In Commit, missing/failing runner yields `E_SMOKE`. |
| smoke_runner (global) | boolean | present, absent | High | Needed when policy requires smoke. |
| smoke_outcome | enum | ok, fail | High | Only relevant if runner present and Commit mode. |
| policy.risks.ownership_strict | boolean | false, true | High | Enforced through preflight gate unless `override_preflight=true`. |
| ownership_oracle (global) | boolean | present, absent | Medium | Needed when ownership_strict=true. |
| attestor (global) | boolean | present, absent | Low | Adds attestation fields on Commit success. |
| attestation_outcome | enum | ok, error | Low | Error → omit attestation fields; apply can still succeed. |
| policy.durability.sidecar_integrity | boolean | true, false | Medium | If false, restore skips payload hash enforcement; combined with `best_effort_restore` affects tolerance. |
| policy.durability.backup_durability | boolean | true, false | Low | Affects emitted `backup_durable` flag in facts; durability fsync is best-effort. |

Constraints:

- IF locking=Required AND mode=Commit THEN lock_manager MUST be present (hard); else expect early `E_LOCKING`.
- IF smoke=Require{..} AND mode=Commit THEN smoke_runner MUST be present; else `E_SMOKE` and auto-rollback policy applies.
- IF mode=DryRun THEN smoke is not executed; attestation is not emitted; timestamps are zeroed (determinism).
- IF override_preflight=false THEN apply enforces preflight STOPs; else proceeds.

## Function: Switchyard::plan_rollback_of(&self, &ApplyReport) -> Plan

Source: `src/api/rollback.rs`

| Axis | Domain | Levels (+boundaries) | Risk | Notes/Constraints |
|---|---|---|---|---|
| policy.apply.capture_restore_snapshot | boolean | false, true | Medium | Enables inversion of `RestoreFromBackup` to `RestoreFromBackup` when true. |
| apply_report.executed_shape | enum | only_symlink, only_restore, mixed | Medium | Determines which inverse actions appear. |

Constraints:

- None hard; function produces best-effort inverse plan given `policy` and `report`.

## Function: Switchyard::prune_backups(&self, &SafePath) -> Result<PruneResult, ApiError>

Source: `src/api/mod.rs` (method body)

| Axis | Domain | Levels (+boundaries) | Risk | Notes/Constraints |
|---|---|---|---|---|
| policy.backup.tag | string | "default", "coreutils", <long>=256 | Low | Tag scopes which artifacts to prune. |
| policy.retention_count_limit | Option<usize> | none, <min>=0, one, many(5) | Medium | Count-based retention. |
| policy.retention_age_limit | Option<Duration> | none, <min>=1s, nominal=1d, <max>=365d | Medium | Age-based retention. |

Constraints:

- None hard beyond path validity; negative envs cover missing directories and I/O errors.

## Type: SafePath::from_rooted(root, candidate) -> Result<SafePath>

Source: `src/types/safepath.rs`

| Axis | Domain | Levels (+boundaries) | Risk | Notes/Constraints |
|---|---|---|---|---|
| root_is_absolute | boolean | true, <invalid>=false | High | Root must be absolute (asserts). |
| candidate_form | enum | relative_normal, absolute_inside_root, absolute_outside_root<invalid>, dotdot<invalid>, curdir_components, unsupported_component<invalid> | High | Enforces SafePath invariants. |
| path_length | range | <min>=empty, short=3 segs, long=255 bytes, <huge>=4096 bytes | Medium | Boundary handling; long/huge covered by env path limits. |
| unicode | boolean | false, true | Low | Accepts unicode in segments. |

Constraints:

- Root must be absolute.
- Candidate must not contain `..` or escape the root.

## Risk Classification Rationale

- High: can cause data loss, failed atomicity, or security gating defects (locking, smoke, ownership strictness, EXDEV policy, rescue requirements).
- Medium: can prevent correct behavior or lead to wrong selection but not destructive (timeouts, retention, preservation inversion, collection sizes).
- Low: ancillary surfaces (attestation presence/outcome, tags, unicode handling) that minimally impact safety but must be tested for completeness.
