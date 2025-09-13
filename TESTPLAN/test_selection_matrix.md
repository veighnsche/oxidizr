# Test Selection Matrix (Switchyard Library)

This document specifies the deterministic selection strategy and the curated scenario set that provides pairwise coverage across all axes per function, with targeted 3-wise coverage for high-risk interactions and boundary/negative overlays.

## Generation Strategy

- Default: pairwise combinations per function across all declared axes in `TESTPLAN/combinatorial_model.json`.
- Escalations to 3-wise for High-risk areas:
  - Apply: `mode × governance.locking × lock_manager`
  - Apply: `apply.exdev × mode × apply.best_effort_restore`
  - Apply: `governance.smoke × smoke_runner × mode`
  - Preflight: `risks.ownership_strict × ownership_oracle × policy.apply.override_preflight`
- Boundary overlays: for each axis with boundary levels (min/max/empty/huge/invalid), add at least one targeted scenario.
- Negative overlays: explicit invalid states and I/O failures (E_LOCKING, E_EXDEV, E_BACKUP_MISSING, E_RESTORE_FAILED, ENOSPC) and contract violations (invalid SafePath).

## Determinism of Selection

- Pairwise seed: 4242
- 3-wise seed: 314159
- Randomized input ordering (where used) seed: 42
- Timestamps: DryRun uses `TS_ZERO` via `logging::redact.rs`; Commit scenarios assert on stable FS state and stable fact fields only.

## Selected Scenarios (human-readable)

Each row below is one scenario ID with explicit axes/levels, environment tags, and rationale. Steps are intentionally high-level (no code).

Columns: ID | Function & Purpose | Axes/Levels | Preconditions/Environment | Steps | Oracles | Tags | Determinism | Cost

| ID | Function & Purpose | Axes/Levels | Preconditions / Environment | Steps | Oracles | Tags | Determinism | Cost |
|---|---|---|---|---|---|---|---|---|
| E2E-PLAN-001 | Plan empty inputs | link_count=min:0; restore_count=min:0; input_order=sorted | Temp root with no files | Build `PlanInput{}`; call `plan` | `Plan.actions=[]`; emits `plan` facts with TS_ZERO | [pairwise] [boundary:min] [tier:Bronze] | seed=42; ts=TS_ZERO | fast; parallel |
| E2E-PLAN-002 | Plan sorts actions deterministically | link_count=many:10; restore_count=one; input_order=random:seed=42; duplicate_targets=false | Temp root; create stub paths | Build mixed `PlanInput`; call `plan` | Sorted by kind then `target.rel`; facts contain per-action rows | [pairwise] [soak-lite] [tier:Silver] | seed=42; ts=TS_ZERO | fast; parallel |
| E2E-PREFLIGHT-001 | Rescue required but unavailable → STOP | rescue.require=true; min_count=nominal:1 | Env: rescue tools absent | Build plan with one action; call `preflight` | `ok=false`; `stops` contains "rescue profile unavailable"; summary fact `E_POLICY` | [negative] [boundary:min] [tier:Silver] | ts=TS_ZERO | fast; parallel |
| E2E-PREFLIGHT-002 | Ownership strict without oracle → STOP | risks.ownership_strict=true; ownership_oracle=absent | No oracle injected | plan→preflight | `ok=false`; `stops` include ownership; summary co-emits `E_OWNERSHIP` | [pairwise] [negative] [tier:Silver] | ts=TS_ZERO | fast; parallel |
| E2E-PREFLIGHT-003 | Preservation required but unsupported | durability.preservation=RequireBasic | Env: target path on FS without preservation | plan→preflight | `ok=false`; stop reason mentions preservation unsupported | [pairwise] [negative] [tier:Silver] | ts=TS_ZERO | fast |
| E2E-APPLY-001 | DryRun symlink ensure | mode=DryRun; governance.locking=Optional; apply.exdev=Fail; override_preflight=true | Default env | plan (one symlink) → apply | Success; `ApplyReport.errors=[]`; facts TS_ZERO; after_kind="symlink" | [pairwise] [tier:Bronze] | ts=TS_ZERO | fast |
| E2E-APPLY-002 | Commit happy path | mode=Commit; locking=Required; lock_manager=present; smoke=Off | File lock manager configured | plan→preflight(ok)→apply | Success; `executed` matches plan; summary no error_id | [pairwise] [tier:Silver] | time=now; ignore volatile fact fields | medium |
| E2E-APPLY-003 | Locking required but no manager → E_LOCKING | mode=Commit; locking=Required; lock_manager=absent; lock_timeout=min:0 | Default env | plan→apply | Failure early; `summary_error_ids` includes `E_LOCKING`; `rolled_back=false` | [negative] [3-wise] [tier:Silver] | time=now | fast |
| E2E-APPLY-004 | Smoke required, runner absent → E_SMOKE + auto-rollback=false | mode=Commit; smoke=Require:auto_rollback=false; smoke_runner=absent | Default env | plan→apply | Failure; `error_id=E_SMOKE`; `rolled_back=false` | [pairwise] [negative] [tier:Silver] | time=now | medium |
| E2E-APPLY-005 | EXDEV degraded fallback used | mode=Commit; apply.exdev=DegradedFallback | Env: set SWITCHYARD_FORCE_EXDEV=1 | plan (symlink)→apply | Success; fact `degraded=true`; after_kind symlink | [3-wise] [env:exdev] [tier:Gold] | time=now | medium |
| E2E-APPLY-006 | Ownership strict gates apply via preflight | mode=Commit; ownership_strict=true; override_preflight=false; ownership_oracle=absent | Default env | plan→apply | Early failure due to preflight STOP; `E_POLICY` in summary | [3-wise] [negative] [tier:Silver] | time=now | fast |
| E2E-APPLY-007 | Attestation present on success | mode=Commit; attestor=present | Default env | plan→apply | Success; summary `attestation` object present | [pairwise] [tier:Gold] | time=now | medium |
| E2E-APPLY-008 | Best-effort restore tolerates missing backup | mode=Commit; best_effort_restore=true | Env: no backups | plan with RestoreFromBackup → apply | Success; no `E_BACKUP_MISSING`; path converges | [pairwise] [negative→allowed] [tier:Silver] | time=now | medium |
| E2E-APPLY-009 | Capture restore snapshot enables inverse | mode=Commit; capture_restore_snapshot=true | Default env | plan with RestoreFromBackup → apply | Success; later `plan_rollback_of` yields `RestoreFromBackup` | [pairwise] [tier:Silver] | time=now | medium |
| E2E-APPLY-010 | Lock timeout high but acquires | mode=Commit; locking=Required; lock_manager=present; lock_timeout=max:10000 | File lock | plan→apply | Success; apply_attempt shows wait_ms <= timeout | [boundary:max] [tier:Silver] | time=now | medium |
| E2E-APPLY-011 | Smoke required, runner present, fails → auto-rollback=true | mode=Commit; smoke=Require:auto_rollback=true; smoke_runner=present; smoke_outcome=fail | Runner injected to fail | plan→apply | Failure; `rolled_back=true`; `E_SMOKE` | [3-wise] [negative] [tier:Gold] | time=now | medium |
| E2E-APPLY-012 | Smoke required, runner present, ok | mode=Commit; smoke=Require:auto_rollback=true; smoke_runner=present; smoke_outcome=ok | Runner injected ok | plan→apply | Success; `rolled_back=false` | [pairwise] [tier:Gold] | time=now | medium |
| E2E-APPLY-013 | Attestation error is tolerated | mode=Commit; attestor=present; attestation_outcome=error | Attestor injected to fail | plan→apply | Success; no `attestation` object | [negative→allowed] [tier:Silver] | time=now | medium |
| E2E-ROLLBACK-001 | Invert only symlink actions | capture_restore_snapshot=false; executed_shape=only_symlink | From ApplyReport | call `plan_rollback_of` | Plan of restores for same targets | [pairwise] [tier:Bronze] | seed=42 | fast |
| E2E-ROLLBACK-002 | Invert previous restore when snapshot captured | capture_restore_snapshot=true; executed_shape=only_restore | From ApplyReport | call `plan_rollback_of` | Plan contains `RestoreFromBackup` | [pairwise] [tier:Silver] | seed=42 | fast |
| E2E-ROLLBACK-003 | Mixed executed actions | capture_restore_snapshot=true; executed_shape=mixed | From ApplyReport | call `plan_rollback_of` | Mixed inverse plan as per policy | [pairwise] [tier:Silver] | seed=42 | fast |
| E2E-PRUNE-001 | Count limit boundary (min=0) | backup.tag=default; retention_count_limit=min:0 | Env: multiple backups | call `prune_backups` | Newest retained; others pruned; counts match | [boundary:min] [tier:Silver] | time=now | fast |
| E2E-PRUNE-002 | Age limit nominal | retention_age_limit=nominal:1d | Env: seeded timestamps | call `prune_backups` | Entries older than 1d pruned; newest kept | [pairwise] [tier:Silver] | time=now | fast |
| E2E-PRUNE-003 | Long tag strings | backup.tag=long:256 | Env: backups exist | call `prune_backups` | Operates by tag; counts correct | [boundary:long] [tier:Bronze] | time=now | fast |
| E2E-SAFEPATH-001 | Reject dotdot | root_is_absolute=true; candidate_form=dotdot:invalid | N/A | call `SafePath::from_rooted` | Error Policy | [negative] [boundary] [tier:Bronze] | seed=42 | fast |
| E2E-SAFEPATH-002 | Accept absolute inside root | candidate_form=absolute_inside_root | N/A | call `from_rooted` | Ok; rel normalized | [pairwise] [tier:Bronze] | seed=42 | fast |
| E2E-SAFEPATH-003 | Reject absolute outside root | candidate_form=absolute_outside_root:invalid | N/A | call `from_rooted` | Error Policy | [negative] [tier:Bronze] | seed=42 | fast |
| E2E-APPLY-014 | ENOSPC during backup/restore path | mode=Commit; durability.backup_durability=true | Env: low disk space | plan→apply | Failure with I/O error; summary `E_POLICY`/generic; no partials | [negative:io] [boundary:low-disk] [tier:Platinum] | time=now | medium |
| E2E-APPLY-015 | Lock contention timeout | mode=Commit; locking=Required; lock_manager=present; lock_timeout=nominal:100 | Env: held lock by rival process | plan→apply | Failure `E_LOCKING`; bounded wait <= timeout | [soak] [negative] [tier:Gold] | time=now | medium |

## Additional Scenarios (coverage completion)

| ID | Function & Purpose | Axes/Levels | Preconditions / Environment | Steps | Oracles | Tags | Determinism | Cost |
|---|---|---|---|---|---|---|---|---|
| E2E-PLAN-003 | Plan handles duplicate targets | duplicate_targets=true; link_count=one; input_order=reverse | Temp root; two links to same target | Build `PlanInput`; call `plan` | Stable sort; actions preserved (no dedupe) | [pairwise] [tier:Bronze] | seed=42; ts=TS_ZERO | fast |
| E2E-PLAN-004 | Plan with huge action set | link_count=huge:1000; restore_count=min:0 | Temp root; generate 1000 links | Build `PlanInput`; call `plan` | Deterministic ordering; performance within budget | [boundary:huge] [tier:Platinum] | seed=42; ts=TS_ZERO | medium |
| E2E-PREFLIGHT-004 | Rescue not required (happy) | rescue.require=false; min_count=min:0 | Env: rescue tools absent | plan→preflight | `ok=true`; summary success | [pairwise] [tier:Bronze] | ts=TS_ZERO | fast |
| E2E-PREFLIGHT-005 | Exec check with large min_count | rescue.require=true; exec_check=true; min_count=huge:100 | Env: only few tools present | plan→preflight | `ok=false`; stop mentions rescue profile | [boundary:huge] [negative] [tier:Silver] | ts=TS_ZERO | fast |
| E2E-PREFLIGHT-006 | Extra mount checks many | apply.extra_mount_checks.count=many:5 | Env: provide 5 mount points | plan→preflight | Rows include notes; summary success/failure per env | [pairwise] [tier:Silver] | ts=TS_ZERO | fast |
| E2E-PREFLIGHT-007 | Long backup tag in preflight | backup.tag=long:256 | Default env | plan→preflight | Rows include tag; success | [boundary:long] [tier:Bronze] | ts=TS_ZERO | fast |
| E2E-APPLY-016 | Restore without capture snapshot | mode=Commit; capture_restore_snapshot=false | Default env; prepare backup | plan (restore)→apply | Success; later rollback plan does not use previous snapshot | [pairwise] [tier:Silver] | time=now | medium |
| E2E-APPLY-017 | Ownership strict with oracle present | mode=Commit; risks.ownership_strict=true; ownership_oracle=present | Oracle injected returns ownership | plan→preflight→apply | Success; summary has provenance; no STOP | [3-wise] [tier:Silver] | time=now | medium |
| E2E-APPLY-018 | DryRun ignores smoke runner | mode=DryRun; smoke=Require:auto_rollback=true; smoke_runner=present | Runner present | plan→apply | Success; smoke not executed; no E_SMOKE | [boundary] [tier:Silver] | ts=TS_ZERO | fast |
| E2E-PRUNE-004 | Prune with coreutils tag | backup.tag=coreutils | Env: multiple tagged backups | call `prune_backups` | Only entries with `coreutils` tag considered | [pairwise] [tier:Bronze] | time=now | fast |
| E2E-PRUNE-005 | Retention none (count/age) | retention_count_limit=none; retention_age_limit=none | Env: multiple backups | call `prune_backups` | No deletions; retained_count==existing | [pairwise] [tier:Bronze] | time=now | fast |
| E2E-PRUNE-006 | Retain only one newest | retention_count_limit=one:1 | Env: >=2 backups | call `prune_backups` | Exactly one retained (newest) | [boundary] [tier:Silver] | time=now | fast |
| E2E-PRUNE-007 | Retain five newest | retention_count_limit=many:5 | Env: >=6 backups | call `prune_backups` | Exactly five retained | [pairwise] [tier:Silver] | time=now | fast |
| E2E-PRUNE-008 | Age limit none with count min=0 | retention_age_limit=none; retention_count_limit=min:0 | Env: multiple backups | call `prune_backups` | Newest retained; rest pruned by count | [pairwise] [boundary:min] [tier:Silver] | time=now | fast |
| E2E-SAFEPATH-004 | Relative normal candidate | candidate_form=relative_normal | N/A | call `from_rooted` | Ok | [pairwise] [tier:Bronze] | seed=42 | fast |
| E2E-SAFEPATH-005 | Curdir normalization | candidate_form=curdir_components | N/A | call `from_rooted` | Ok; rel normalized (no ./) | [boundary] [tier:Bronze] | seed=42 | fast |
| E2E-SAFEPATH-006 | Root not absolute (invalid) | root_is_absolute=invalid:false | N/A | call `from_rooted` | Panic or Error per assert | [negative] [tier:Bronze] | seed=42 | fast |
| E2E-PLAN-005 | Plan with many restore actions | restore_count=many:10; link_count=min:0 | Temp root; create 10 restore targets | Build `PlanInput`; call `plan` | Actions sorted deterministically | [boundary] [tier:Silver] | seed=42; ts=TS_ZERO | fast |
| E2E-PREFLIGHT-008 | One extra mount check | apply.extra_mount_checks.count=one | Env: provide 1 mount point | plan→preflight | Row includes mount check note | [pairwise] [tier:Bronze] | ts=TS_ZERO | fast |
| E2E-PREFLIGHT-009 | Empty backup tag | backup.tag=empty | Default env | plan→preflight | Succeeds; tag present in facts | [boundary] [tier:Bronze] | ts=TS_ZERO | fast |
| E2E-APPLY-019 | EXDEV with Fail policy → error | mode=Commit; apply.exdev=Fail | Env: SWITCHYARD_FORCE_EXDEV=1 | plan (symlink)→apply | Failure; per-action error_id=E_EXDEV | [3-wise] [env:exdev] [negative] [tier:Gold] | time=now | medium |
| E2E-APPLY-020 | Sidecar integrity disabled tolerates tamper | mode=Commit; policy.durability.sidecar_integrity=false | Env: tampered backup payload hash | plan (restore)→apply | Success; `sidecar_integrity_verified` may be false but no error | [pairwise] [negative→allowed] [tier:Gold] | time=now | medium |
| E2E-APPLY-021 | Backup durability flag off | mode=Commit; policy.durability.backup_durability=false | Default env | plan→apply | Success; facts have `backup_durable=false` | [pairwise] [tier:Bronze] | time=now | fast |
| E2E-PRUNE-009 | Age limit max (365d) | retention_age_limit=max:365d | Env: seeded timestamps | call `prune_backups` | Only >365d pruned; newest kept | [boundary:max] [tier:Silver] | time=now | fast |
| E2E-SAFEPATH-007 | Unsupported component invalid | candidate_form=unsupported_component:invalid | N/A | call `from_rooted` | Error InvalidPath | [negative] [tier:Bronze] | seed=42 | fast |
| E2E-SAFEPATH-008 | Empty candidate | path_length=empty | N/A | call `from_rooted` | Ok (rel="") within root | [boundary:min] [tier:Bronze] | seed=42 | fast |
| E2E-SAFEPATH-009 | Unicode segments | unicode=true | N/A | call `from_rooted` | Ok; rel preserves unicode | [pairwise] [tier:Bronze] | seed=42 | fast |
| E2E-PLAN-006 | Plan single link | link_count=one; restore_count=min:0; input_order=sorted; duplicate_targets=false | Temp root | Build single-link `PlanInput`; call `plan` | One EnsureSymlink action; sorted self | [pairwise] [tier:Bronze] | seed=42; ts=TS_ZERO | fast |
| E2E-PREFLIGHT-010 | Exec check disabled baseline | rescue.exec_check=false | Default env | plan→preflight | `ok` per other axes; no exec probe | [pairwise] [tier:Bronze] | ts=TS_ZERO | fast |
| E2E-PREFLIGHT-011 | Coreutils backup tag | backup.tag=coreutils | Default env | plan→preflight | Rows carry tag; success | [pairwise] [tier:Bronze] | ts=TS_ZERO | fast |
| E2E-PRUNE-010 | Age limit min (1s) | retention_age_limit=min:1s | Env: seeded timestamps just over 1s | call `prune_backups` | Entries older than 1s pruned; newest kept | [boundary:min] [tier:Silver] | time=now | fast |
| E2E-SAFEPATH-010 | Short path (3 segments) | path_length=short:3segs | N/A | call `from_rooted` | Ok; rel has 3 segments | [boundary] [tier:Bronze] | seed=42 | fast |
| E2E-SAFEPATH-011 | Long path (255 bytes) | path_length=long:255bytes | N/A | call `from_rooted` | Ok; rel preserved | [boundary:long] [tier:Gold] | seed=42 | medium |
| E2E-SAFEPATH-012 | Huge path (4096 bytes) | path_length=huge:4096bytes | N/A | call `from_rooted` | Ok or Error per OS limit; documented | [boundary:huge] [tier:Platinum] | seed=42 | medium |
| E2E-APPLY-022 | Crash between backup and rename | mode=Commit; apply.exdev=Fail | Env: crash injection at swap step | plan (symlink)→apply | Process killed deterministically; on rerun with same plan, system converges and no temp files persist | [negative:fault] [soak] [tier:Platinum] | time=now | medium |

Notes:

- [pairwise] rows are part of the seed=4242 pairwise set; [3-wise] rows are additional targeted cases for High-risk interactions.
- [boundary:*] cover specific boundary levels for at least one axis each.
- [negative:*] denote contract or environmental failure paths.
- [soak] mark scenarios suitable for stress or contention rotation.
