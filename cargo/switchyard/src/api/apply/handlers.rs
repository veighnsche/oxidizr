use serde_json::json;
use uuid::Uuid;

use crate::constants::FSYNC_WARN_MS;
use crate::logging::{AuditSink, FactsEmitter};
use crate::types::ids::action_id;
use crate::types::Action;

use super::audit_fields::{insert_hashes, maybe_warn_fsync};
use std::time::Instant;
use crate::api::errors::{exit_code_for, id_str, ErrorId};
use crate::fs::meta::{kind_of, resolve_symlink_target, sha256_hex_of};
use crate::logging::audit::{emit_apply_attempt, emit_apply_result, ensure_provenance, AuditCtx};

/// Handle an EnsureSymlink action: perform the operation and emit per-action facts.
/// Returns (executed_action_if_success, error_message_if_failure).
pub(crate) fn handle_ensure_symlink<E: FactsEmitter, A: AuditSink>(
    api: &super::super::Switchyard<E, A>,
    tctx: &AuditCtx,
    pid: &Uuid,
    act: &Action,
    idx: usize,
    dry: bool,
) -> (Option<Action>, Option<String>, super::PerfAgg) {
    let (source, target) = match act {
        Action::EnsureSymlink { source, target } => (source, target),
        _ => unreachable!("expected EnsureSymlink"),
    };

    let _aid = action_id(pid, act, idx);
    // Attempt fact
    emit_apply_attempt(
        tctx,
        "success",
        json!({
            "action_id": _aid.to_string(),
            "path": target.as_path().display().to_string(),
            "safepath_validation": "success",
            "backup_durable": api.policy.require_backup_durability,
        }),
    );

    let degraded_used: bool;
    let mut fsync_ms: u64 = 0;
    let before_kind = kind_of(&target.as_path());
    // Compute before/after hashes (time the operation)
    let th0 = Instant::now();
    let before_hash = match resolve_symlink_target(&target.as_path()) {
        Some(p) => sha256_hex_of(&p),
        None => sha256_hex_of(&target.as_path()),
    };
    let after_hash = sha256_hex_of(&source.as_path());
    let hash_ms = th0.elapsed().as_millis() as u64;
    match crate::fs::replace_file_with_symlink(
        &source,
        &target,
        dry,
        api.policy.allow_degraded_fs,
        &api.policy.backup_tag,
    ) {
        Ok((d, ms)) => {
            degraded_used = d;
            fsync_ms = ms;
        }
        Err(e) => {
            // Map to Silver-tier error ids for atomic swap/exdev
            let emsg = e.to_string();
            let id = if emsg.contains("sidecar write failed") {
                ErrorId::E_POLICY
            } else {
                match e.raw_os_error() {
                    Some(code) if code == libc::EXDEV => ErrorId::E_EXDEV,
                    _ => ErrorId::E_ATOMIC_SWAP,
                }
            };
            let msg = format!(
                "symlink {} -> {} failed: {}",
                source.as_path().display(),
                target.as_path().display(),
                e
            );
            // Emit result with failure now
            let mut extra = json!({
                "action_id": _aid.to_string(),
                "path": target.as_path().display().to_string(),
                // On failure explicitly record degraded=false and reason when EXDEV
                "degraded": Some(false),
                "degraded_reason": if matches!(id, ErrorId::E_EXDEV) { Some("exdev_fallback") } else { None },
                "error_detail": if matches!(id, ErrorId::E_EXDEV) { Some("exdev_fallback_failed") } else { None },
                "duration_ms": fsync_ms,
                "before_kind": before_kind,
                "after_kind": if dry { "symlink".to_string() } else { kind_of(&target.as_path()) },
            });
            ensure_provenance(&mut extra);
            insert_hashes(&mut extra, &before_hash, &after_hash);
            let obj = extra.as_object_mut().unwrap();
            obj.insert("error_id".to_string(), json!(id_str(id)));
            obj.insert("exit_code".to_string(), json!(exit_code_for(id)));
            emit_apply_result(tctx, "failure", extra);
            return (None, Some(msg), super::PerfAgg { hash_ms, backup_ms: 0, swap_ms: fsync_ms });
        }
    }

    // Success path: emit result
    let mut extra = json!({
        "action_id": _aid.to_string(),
        "path": target.as_path().display().to_string(),
        "degraded": if degraded_used { Some(true) } else { None },
        "degraded_reason": if degraded_used { Some("exdev_fallback") } else { None },
        "duration_ms": fsync_ms,
        "before_kind": before_kind,
        "after_kind": if dry { "symlink".to_string() } else { kind_of(&target.as_path()) },
        "backup_durable": api.policy.require_backup_durability,
    });
    ensure_provenance(&mut extra);
    insert_hashes(&mut extra, &before_hash, &after_hash);
    maybe_warn_fsync(&mut extra, fsync_ms, FSYNC_WARN_MS);
    emit_apply_result(tctx, "success", extra);

    (Some(act.clone()), None, super::PerfAgg { hash_ms, backup_ms: 0, swap_ms: fsync_ms })
}

/// Handle a RestoreFromBackup action: perform restore and emit per-action facts.
/// Returns (executed_action_if_success, error_message_if_failure).
pub(crate) fn handle_restore<E: FactsEmitter, A: AuditSink>(
    api: &super::super::Switchyard<E, A>,
    tctx: &AuditCtx,
    pid: &Uuid,
    act: &Action,
    idx: usize,
    dry: bool,
) -> (Option<Action>, Option<String>, super::PerfAgg) {
    let target = match act {
        Action::RestoreFromBackup { target } => target,
        _ => unreachable!("expected RestoreFromBackup"),
    };
    let _aid = action_id(pid, act, idx);

    emit_apply_attempt(
        tctx,
        "success",
        json!({
            "action_id": _aid.to_string(),
            "path": target.as_path().display().to_string(),
            "safepath_validation": "success",
            "backup_durable": api.policy.require_backup_durability,
        }),
    );

    let before_kind = kind_of(&target.as_path());
    let mut used_prev = false;
    let mut backup_ms = 0u64;
    if !dry && api.policy.capture_restore_snapshot {
        let tb0 = Instant::now();
        let _ = crate::fs::create_snapshot(&target.as_path(), &api.policy.backup_tag);
        backup_ms = tb0.elapsed().as_millis() as u64;
        used_prev = true;
    }
    let force = api.policy.force_restore_best_effort || !api.policy.require_sidecar_integrity;
    // Pre-compute sidecar integrity verification (best-effort) before restore
    let th0 = Instant::now();
    let integrity_verified = (|| {
        let pair = if used_prev {
            crate::fs::backup::find_previous_backup_and_sidecar(&target.as_path(), &api.policy.backup_tag)
        } else {
            crate::fs::backup::find_latest_backup_and_sidecar(&target.as_path(), &api.policy.backup_tag)
        }?;
        let (backup_opt, sc_path) = pair;
        let sc = crate::fs::backup::read_sidecar(&sc_path).ok()?;
        if let (Some(backup), Some(hash)) = (backup_opt, sc.payload_hash) {
            let actual = crate::fs::meta::sha256_hex_of(&backup)?;
            Some(actual == hash)
        } else {
            None
        }
    })();
    let mut hash_ms = th0.elapsed().as_millis() as u64;

    let restore_res = if used_prev {
        crate::fs::restore_file_prev(&target, dry, force, &api.policy.backup_tag)
    } else {
        crate::fs::restore_file(&target, dry, force, &api.policy.backup_tag)
    };
    match restore_res {
        Ok(()) => {
            // success
        }
        Err(mut e) => {
            // If we tried previous and it was NotFound (no previous), fall back to latest
            if used_prev && e.kind() == std::io::ErrorKind::NotFound {
                if let Err(e2) =
                    crate::fs::restore_file(&target, dry, force, &api.policy.backup_tag)
                {
                    e = e2;
                } else {
                    // success on fallback
                    let decision = "success";
                    let mut extra = json!({
                        "action_id": _aid.to_string(),
                        "path": target.as_path().display().to_string(),
                        "before_kind": before_kind,
                        "after_kind": if dry { before_kind.clone() } else { kind_of(&target.as_path()) },
                    });
                    if let Some(iv) = integrity_verified {
                        if let Some(obj) = extra.as_object_mut() { obj.insert("sidecar_integrity_verified".into(), json!(iv)); }
                    }
                    ensure_provenance(&mut extra);
                    emit_apply_result(tctx, decision, extra);
                    return (Some(act.clone()), None, super::PerfAgg { hash_ms, backup_ms, swap_ms: 0 });
                }
            }
            use std::io::ErrorKind;
            let id = match e.kind() {
                ErrorKind::NotFound => ErrorId::E_BACKUP_MISSING,
                _ => ErrorId::E_RESTORE_FAILED,
            };
            let msg = format!("restore {} failed: {}", target.as_path().display(), e);
            let decision = "failure";
            let mut extra = json!({
                "action_id": _aid.to_string(),
                "path": target.as_path().display().to_string(),
                "before_kind": before_kind,
                "after_kind": if dry { before_kind.clone() } else { kind_of(&target.as_path()) },
            });
            if let Some(iv) = integrity_verified {
                if let Some(obj) = extra.as_object_mut() { obj.insert("sidecar_integrity_verified".into(), json!(iv)); }
            }
            ensure_provenance(&mut extra);
            let obj = extra.as_object_mut().unwrap();
            obj.insert("error_id".to_string(), json!(id_str(id)));
            obj.insert("exit_code".to_string(), json!(exit_code_for(id)));
            emit_apply_result(tctx, decision, extra);
            return (None, Some(msg), super::PerfAgg { hash_ms, backup_ms, swap_ms: 0 });
        }
    }

    // Success path
    let decision = "success";
    let mut extra = json!({
        "action_id": _aid.to_string(),
        "path": target.as_path().display().to_string(),
        "before_kind": before_kind,
        "after_kind": if dry { before_kind } else { kind_of(&target.as_path()) },
        "backup_durable": api.policy.require_backup_durability,
    });
    if let Some(iv) = integrity_verified {
        if let Some(obj) = extra.as_object_mut() { obj.insert("sidecar_integrity_verified".into(), json!(iv)); }
    }
    ensure_provenance(&mut extra);
    emit_apply_result(tctx, decision, extra);

    (Some(act.clone()), None, super::PerfAgg { hash_ms, backup_ms, swap_ms: 0 })
}
