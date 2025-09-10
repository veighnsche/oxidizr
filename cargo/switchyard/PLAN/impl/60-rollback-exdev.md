# Rollback, EXDEV Fallback, and Partial Restoration (Planning Only)

Defines the rollback strategy, cross-filesystem (EXDEV) degraded fallback, and partial restoration reporting.

References:

- SPEC: `cargo/switchyard/SPEC/SPEC.md §2.2 Rollback`, `§2.10 Filesystems & Degraded Mode`
- Requirements: `REQ-R1..R5`, `REQ-F1..F3`, `REQ-C2`

## Goals

- Automatic reverse-order rollback on any apply failure. (REQ-R4)
- Rollback is complete and idempotent; repeated rollback is a no-op. (REQ-R1..R3)
- On rollback failure, record partial restoration state in facts. (REQ-R5)
- On cross-filesystem, use safe copy + fsync + rename fallback, gated by policy; facts record `degraded=true` when used. (REQ-F1..F2)

## Rust-like Pseudocode (non-compilable)

```rust
// Planning-only pseudocode

struct Backup { path: SafePath, tmp: SafePath, before_hash: String }

fn apply(plan: &Plan, mode: ApplyMode, adapters: &Adapters) -> ApplyReport {
    let mut backups: Vec<Backup> = vec![];
    // ... lock acquisition and attempt facts omitted (see 50-locking-concurrency.md)

    for a in &plan.actions {
        emit_fact(Fact{ stage: ApplyAttempt, action_id: Some(a.action_id), path: Some(a.path.abs()), ..Default });
        if mode == DryRun { emit_success(a, dry_run=true); continue; }

        let result = match a.kind {
            ReplaceSymlink => replace_symlink(a, &mut backups),
            RestoreFromBackup => restore_backup(a),
            Skip => Ok(())
        };

        if let Err(err) = result {
            emit_failure(a, &err);
            let partial = rollback(&mut backups);        // reverse order, best effort
            record_partial_restoration(&partial);        // REQ-R5
            return ApplyReport{ decision: Failure, partial_restoration: partial, cause: err.kind };
        }
        emit_success(a, dry_run=false);
    }

    // ... smoke tests and attestation omitted (see 40-facts-logging.md)
    ApplyReport{ decision: Success, partial_restoration: None, cause: None }
}

fn replace_symlink(a: &Action, backups: &mut Vec<Backup>) -> Result<(), Error> {
    // Enforce TOCTOU-safe sequence; SafePath invariant holds
    let parent = a.path.open_parent_dir_no_follow()?;
    let staged = stage_new_symlink(a)?;                 // temp symlink under parent dir
    let backup  = backup_existing_if_any(a.path)?;      // capture before_hash

    // Atomic rename with EXDEV fallback
    atomic_rename(&parent, &staged, &a.path)?;          // REQ-F1
    fsync_parent(&parent)?;                             // ≤50ms; REQ-BND1

    if let Some(b) = backup { backups.push(b); }
    Ok(())
}

fn atomic_rename(parent: &DirHandle, staged: &SafePath, target: &SafePath) -> Result<(), Error> {
    if same_filesystem(staged, target) {
        renameat(staged, target).map_err(|_| Error{kind:E_ATOMIC_SWAP, msg:"rename"})?;
    } else {
        if !policy.allow_degraded_fs { return Err(Error{kind:E_EXDEV, msg:"cross-fs disallowed"}); }
        emit_fact(Fact{ stage: ApplyAttempt, decision: Warn, degraded: Some(true), ..Default });
        copy_file(staged, tmp_for(target))?;            // copy
        fsync(tmp_for(target))?;                        // sync
        renameat(tmp_for(target), target)?;             // visible switch
    }
    Ok(())
}

fn rollback(backups: &mut Vec<Backup>) -> PartialRestoration {
    let mut partial = PartialRestoration::new();
    while let Some(b) = backups.pop() {                 // reverse order
        match restore_one(&b) {
            Ok(()) => emit_fact(Fact{ stage: Rollback, decision: Success, path: Some(b.path.abs()), ..Default }),
            Err(e) => {
                emit_fact(Fact{ stage: Rollback, decision: Failure, path: Some(b.path.abs()), exit_code: Some(to_exit_code(&e.kind)), ..Default });
                partial.add_failed(b.path.clone(), e.kind);
            }
        }
    }
    partial
}

fn restore_backup(a: &Action) -> Result<(), Error> {
    let parent = a.path.open_parent_dir_no_follow()?;
    let src = locate_backup_for(&a.path)?;
    atomic_rename(&parent, &src, &a.path)?;
    fsync_parent(&parent)?;
    Ok(())
}
```

## Partial Restoration Facts

- A `PartialRestoration` summary fact MUST list any paths that failed to restore and guidance fields for operator recovery. (REQ-R5)
- Emit per-step rollback facts (stage=`rollback`) with `decision=success|failure` and `exit_code` when applicable.

## Degraded Mode Policy & Telemetry

- If `allow_degraded_fs=true` and EXDEV fallback is used, facts MUST include `degraded=true` (stage=`apply.attempt` and/or `apply.result`). (REQ-F2)
- If `allow_degraded_fs=false`, EXDEV path MUST fail with `E_EXDEV` and no visible change occurs. (REQ-F2)

## Tests & Evidence

- BDD: `atomic_swap.feature :: Cross-filesystem EXDEV fallback`, `Automatic rollback on mid-plan failure`.
- Property: `IdempotentRollback`, `AtomicReplace`.
- Golden fixtures: rollback facts include partial restoration entries when simulated failures occur.
