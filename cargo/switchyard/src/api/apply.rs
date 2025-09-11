//! api/apply.rs — extracted apply() implementation

use std::time::Instant;

use base64::Engine;
use serde_json::json;

use crate::fs;
use crate::logging::ts_for_mode;
use crate::logging::{AuditSink, FactsEmitter};
use crate::types::ids::{action_id, plan_id};
use crate::types::{Action, ApplyMode, ApplyReport, Plan};

use super::fs_meta::{resolve_symlink_target, sha256_hex_of};
use super::audit::{emit_apply_attempt, emit_apply_result, emit_rollback_step, AuditCtx, AuditMode};

pub(super) fn run<E: FactsEmitter, A: AuditSink>(
    api: &super::Switchyard<E, A>,
    plan: &Plan,
    mode: ApplyMode,
) -> ApplyReport {
    let t0 = Instant::now();
    let mut executed: Vec<Action> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut rollback_errors: Vec<String> = Vec::new();
    let mut rolled_back = false;
    let dry = matches!(mode, ApplyMode::DryRun);
    let pid = plan_id(plan);
    let ts_now = ts_for_mode(&mode);

    // Audit context
    let tctx = AuditCtx::new(
        &api.facts as &dyn FactsEmitter,
        pid.to_string(),
        ts_now.clone(),
        AuditMode { dry_run: dry, redact: dry },
    );

    // Locking (optional in dev/test): acquire process lock with bounded wait; emit telemetry via apply.attempt
    let mut lock_wait_ms: Option<u64> = None;
    let mut _lock_guard: Option<Box<dyn crate::adapters::lock::LockGuard>> = None;
    if let Some(mgr) = &api.lock {
        let lt0 = Instant::now();
        match mgr.acquire_process_lock(api.lock_timeout_ms) {
            Ok(guard) => {
                lock_wait_ms = Some(lt0.elapsed().as_millis() as u64);
                _lock_guard = Some(guard);
            }
            Err(e) => {
                emit_apply_attempt(&tctx, "failure", json!({
                    "lock_wait_ms": lock_wait_ms,
                    "error": e.to_string(),
                }));
                let duration_ms = t0.elapsed().as_millis() as u64;
                return ApplyReport {
                    executed,
                    duration_ms,
                    errors: vec![format!("lock: {}", e)],
                    plan_uuid: Some(pid),
                    rolled_back,
                    rollback_errors,
                };
            }
        }
    } else {
        emit_apply_attempt(&tctx, "warn", json!({
            "no_lock_manager": true,
        }));
    }

    // Minimal Facts v1: apply attempt summary (include lock_wait_ms when present)
    emit_apply_attempt(&tctx, "success", json!({
        "lock_wait_ms": lock_wait_ms,
    }));

    for (idx, act) in plan.actions.iter().enumerate() {
        let _aid = action_id(&pid, act, idx);
        match act {
            Action::EnsureSymlink { source, target } => {
                // Minimal Facts v1: per-action attempt
                emit_apply_attempt(&tctx, "success", json!({
                    "action_id": _aid.to_string(),
                    "path": target.as_path().display().to_string(),
                }));
                let mut degraded_used = false;
                let mut fsync_ms: u64 = 0;
                // Compute before/after hashes
                let before_hash = match resolve_symlink_target(&target.as_path()) {
                    Some(p) => sha256_hex_of(&p),
                    None => sha256_hex_of(&target.as_path()),
                };
                let after_hash = sha256_hex_of(&source.as_path());
                match fs::replace_file_with_symlink(
                    &source.as_path(),
                    &target.as_path(),
                    dry,
                    api.policy.allow_degraded_fs,
                    &api.policy.backup_tag,
                ) {
                    Ok((d, ms)) => {
                        degraded_used = d;
                        fsync_ms = ms;
                        executed.push(act.clone());
                    }
                    Err(e) => errors.push(format!(
                        "symlink {} -> {} failed: {}",
                        source.as_path().display(),
                        target.as_path().display(),
                        e
                    )),
                }
                // Minimal Facts v1: per-action result
                let decision = if errors.is_empty() { "success" } else { "failure" };
                let mut extra = json!({
                    "action_id": _aid.to_string(),
                    "path": target.as_path().display().to_string(),
                    "degraded": if degraded_used { Some(true) } else { None },
                    "duration_ms": fsync_ms,
                });
                if let Some(bh) = before_hash.as_ref() {
                    let obj = extra.as_object_mut().unwrap();
                    obj.insert("hash_alg".to_string(), json!("sha256"));
                    obj.insert("before_hash".to_string(), json!(bh));
                }
                if let Some(ah) = after_hash.as_ref() {
                    let obj = extra.as_object_mut().unwrap();
                    obj.insert("hash_alg".to_string(), json!("sha256"));
                    obj.insert("after_hash".to_string(), json!(ah));
                }
                {
                    let obj = extra.as_object_mut().unwrap();
                    obj.insert(
                        "provenance".to_string(),
                        json!({
                            "helper": "",
                            "env_sanitized": true
                        }),
                    );
                }
                if errors.is_empty() && fsync_ms > 50 {
                    let obj = extra.as_object_mut().unwrap();
                    obj.insert("severity".to_string(), json!("warn"));
                }
                emit_apply_result(&tctx, decision, extra);
            }
            Action::RestoreFromBackup { target } => {
                // Minimal Facts v1: per-action attempt
                emit_apply_attempt(&tctx, "success", json!({
                    "action_id": _aid.to_string(),
                    "path": target.as_path().display().to_string(),
                }));
                match fs::restore_file(
                    &target.as_path(),
                    dry,
                    api.policy.force_restore_best_effort,
                    &api.policy.backup_tag,
                ) {
                    Ok(()) => executed.push(act.clone()),
                    Err(e) => errors.push(format!(
                        "restore {} failed: {}",
                        target.as_path().display(),
                        e
                    )),
                }
                // Minimal Facts v1: per-action result
                let decision = if errors.is_empty() { "success" } else { "failure" };
                let mut extra = json!({
                    "action_id": _aid.to_string(),
                    "path": target.as_path().display().to_string(),
                });
                {
                    let obj = extra.as_object_mut().unwrap();
                    obj.insert(
                        "provenance".to_string(),
                        json!({
                            "helper": "",
                            "env_sanitized": true
                        }),
                    );
                }
                emit_apply_result(&tctx, decision, extra);
            }
        }

        // On first failure, attempt reverse-order rollback for already executed actions.
        if !errors.is_empty() {
            if !dry {
                rolled_back = true;
                for prev in executed.iter().rev() {
                    match prev {
                        Action::EnsureSymlink { source: _source, target } => {
                            match fs::restore_file(
                                &target.as_path(),
                                dry,
                                api.policy.force_restore_best_effort,
                                &api.policy.backup_tag,
                            ) {
                                Ok(()) => {
                                    emit_rollback_step(&tctx, "success", &target.as_path().display().to_string());
                                }
                                Err(e) => {
                                    rollback_errors.push(format!(
                                        "rollback restore {} failed: {}",
                                        target.as_path().display(),
                                        e
                                    ));
                                    emit_rollback_step(&tctx, "failure", &target.as_path().display().to_string());
                                }
                            }
                        }
                        Action::RestoreFromBackup { .. } => {
                            // No reliable inverse without prior state capture; record informational error.
                            rollback_errors.push(
                                "rollback of RestoreFromBackup not supported (no prior state)".to_string(),
                            );
                            emit_rollback_step(&tctx, "failure", "");
                        }
                    }
                }
            }
            break;
        }
    }

    // Optional smoke tests post-apply (only in Commit mode)
    if errors.is_empty() && !dry {
        if let Some(smoke) = &api.smoke {
            if smoke.run(plan).is_err() {
                errors.push("smoke tests failed".to_string());
                if !api.policy.disable_auto_rollback {
                    rolled_back = true;
                    for prev in executed.iter().rev() {
                        match prev {
                            Action::EnsureSymlink { source: _s, target } => {
                                let _ = fs::restore_file(
                                    &target.as_path(),
                                    dry,
                                    api.policy.force_restore_best_effort,
                                    &api.policy.backup_tag,
                                )
                                .map_err(|e| {
                                    rollback_errors.push(format!(
                                        "rollback restore {} failed: {}",
                                        target.as_path().display(),
                                        e
                                    ))
                                });
                            }
                            Action::RestoreFromBackup { .. } => {
                                rollback_errors.push("rollback of RestoreFromBackup not supported (no prior state)".to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Final apply.result summary (after smoke tests/rollback)
    let decision = if errors.is_empty() { "success" } else { "failure" };
    let mut fields = json!({});
    // Optional attestation on success, non-dry-run
    if errors.is_empty() && !dry {
        if let Some(att) = &api.attest {
            let bundle: Vec<u8> = Vec::new(); // TODO: real bundle
            if let Ok(sig) = att.sign(&bundle) {
                let sig_b64 = base64::engine::general_purpose::STANDARD.encode(sig.0.clone());
                let att_json = json!({
                    "sig_alg": "ed25519",
                    "signature": sig_b64,
                    "bundle_hash": "", // TODO: sha256 of bundle
                    "public_key_id": "", // TODO
                });
                // Merge attestation into fields
                let obj = fields.as_object_mut().unwrap();
                obj.insert("attestation".to_string(), att_json);
            }
        }
    }
    // we already include ts/stage in helper
    emit_apply_result(&tctx, decision, fields);

    // Compute total duration
    let duration_ms = t0.elapsed().as_millis() as u64;
    ApplyReport {
        executed,
        duration_ms,
        errors,
        plan_uuid: Some(pid),
        rolled_back,
        rollback_errors,
    }
}
