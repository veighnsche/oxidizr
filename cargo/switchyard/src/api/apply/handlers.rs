use serde_json::json;
use uuid::Uuid;

use crate::constants::FSYNC_WARN_MS;
use crate::logging::{AuditSink, FactsEmitter};
use crate::types::ids::action_id;
use crate::types::Action;

use crate::fs::meta::{kind_of, resolve_symlink_target, sha256_hex_of};
use crate::logging::audit::{emit_apply_attempt, emit_apply_result, ensure_provenance, AuditCtx};
use super::audit_fields::{insert_hashes, maybe_warn_fsync};
use crate::api::errors::{exit_code_for, id_str, ErrorId};

/// Handle an EnsureSymlink action: perform the operation and emit per-action facts.
/// Returns (executed_action_if_success, error_message_if_failure).
pub(crate) fn handle_ensure_symlink<E: FactsEmitter, A: AuditSink>(
    api: &super::super::Switchyard<E, A>,
    tctx: &AuditCtx,
    pid: &Uuid,
    act: &Action,
    idx: usize,
    dry: bool,
) -> (Option<Action>, Option<String>) {
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
        }),
    );

    let degraded_used: bool;
    let mut fsync_ms: u64 = 0;
    let before_kind = kind_of(&target.as_path());
    // Compute before/after hashes
    let before_hash = match resolve_symlink_target(&target.as_path()) {
        Some(p) => sha256_hex_of(&p),
        None => sha256_hex_of(&target.as_path()),
    };
    let after_hash = sha256_hex_of(&source.as_path());
    match crate::fs::replace_file_with_symlink(
        &source.as_path(),
        &target.as_path(),
        dry,
        api.policy.allow_degraded_fs,
        &api.policy.backup_tag,
    ) {
        Ok((d, ms)) => { degraded_used = d; fsync_ms = ms; }
        Err(e) => {
            // Map to Silver-tier error ids for atomic swap/exdev
            let emsg = e.to_string();
            let id = if emsg.contains("sidecar write failed") { ErrorId::E_POLICY } else { match e.raw_os_error() { Some(code) if code == libc::EXDEV => ErrorId::E_EXDEV, _ => ErrorId::E_ATOMIC_SWAP, } };
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
                "degraded": None::<bool>,
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
            return (None, Some(msg));
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
    });
    ensure_provenance(&mut extra);
    insert_hashes(&mut extra, &before_hash, &after_hash);
    maybe_warn_fsync(&mut extra, fsync_ms, FSYNC_WARN_MS);
    emit_apply_result(tctx, "success", extra);

    (Some(act.clone()), None)
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
) -> (Option<Action>, Option<String>) {
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
        }),
    );

    let before_kind = kind_of(&target.as_path());
    match crate::fs::restore_file(
        &target.as_path(),
        dry,
        api.policy.force_restore_best_effort,
        &api.policy.backup_tag,
    ) {
        Ok(()) => {
            // success
        }
        Err(e) => {
            use std::io::ErrorKind;
            let id = match e.kind() { ErrorKind::NotFound => ErrorId::E_BACKUP_MISSING, _ => ErrorId::E_RESTORE_FAILED };
            let msg = format!("restore {} failed: {}", target.as_path().display(), e);
            let decision = "failure";
            let mut extra = json!({
                "action_id": _aid.to_string(),
                "path": target.as_path().display().to_string(),
                "before_kind": before_kind,
                "after_kind": if dry { before_kind.clone() } else { kind_of(&target.as_path()) },
            });
            ensure_provenance(&mut extra);
            let obj = extra.as_object_mut().unwrap();
            obj.insert("error_id".to_string(), json!(id_str(id)));
            obj.insert("exit_code".to_string(), json!(exit_code_for(id)));
            emit_apply_result(tctx, decision, extra);
            return (None, Some(msg));
        }
    }

    // Success path
    let decision = "success";
    let mut extra = json!({
        "action_id": _aid.to_string(),
        "path": target.as_path().display().to_string(),
        "before_kind": before_kind,
        "after_kind": if dry { before_kind } else { kind_of(&target.as_path()) },
    });
    ensure_provenance(&mut extra);
    emit_apply_result(tctx, decision, extra);

    (Some(act.clone()), None)
}
