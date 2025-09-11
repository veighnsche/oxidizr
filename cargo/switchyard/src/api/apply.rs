//! Apply stage: executes plan actions with atomic symlink swap, backup/restore, and rollback.
//!
//! Side-effects:
//! - Emits Minimal Facts v1 for `apply.attempt` and `apply.result` per action, plus a summary.
//! - Enforces locking policy and maps failures to `E_LOCKING` with bounded wait.
//! - Enforces policy gating (unless `override_preflight=true`).
//! - Optionally runs smoke tests post-apply and triggers auto-rollback on failures.
//! - Optionally emits an attestation bundle on success.

use std::time::Instant;

use base64::Engine;
use serde_json::json;

use crate::fs;
use crate::logging::ts_for_mode;
use crate::logging::{AuditSink, FactsEmitter};
use crate::types::ids::{action_id, plan_id};
use crate::types::{Action, ApplyMode, ApplyReport, Plan};
use crate::constants::FSYNC_WARN_MS;

use super::fs_meta::{resolve_symlink_target, sha256_hex_of, kind_of};
use super::audit::{emit_apply_attempt, emit_apply_result, emit_rollback_step, ensure_provenance, AuditCtx, AuditMode};
use super::errors::{ErrorId, exit_code_for};

pub(crate) fn run<E: FactsEmitter, A: AuditSink>(
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

    // Locking (required by default in Commit): acquire process lock with bounded wait; emit telemetry via apply.attempt
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
                lock_wait_ms = Some(lt0.elapsed().as_millis() as u64);
                emit_apply_attempt(&tctx, "failure", json!({
                    "lock_wait_ms": lock_wait_ms,
                    "error": e.to_string(),
                    "error_id": "E_LOCKING",
                    "exit_code": 30
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
        if !dry {
            // Enforce by default unless explicitly allowed through policy, or when require_lock_manager is set.
            let must_fail = api.policy.require_lock_manager || !api.policy.allow_unlocked_commit;
            if must_fail {
                emit_apply_attempt(&tctx, "failure", json!({
                    "error_id": "E_LOCKING",
                    "exit_code": 30,
                }));
                let duration_ms = t0.elapsed().as_millis() as u64;
                return ApplyReport {
                    executed,
                    duration_ms,
                    errors: vec!["lock manager required in Commit mode".to_string()],
                    plan_uuid: Some(pid),
                    rolled_back,
                    rollback_errors,
                };
            } else {
                emit_apply_attempt(&tctx, "warn", json!({
                    "no_lock_manager": true,
                }));
            }
        } else {
            emit_apply_attempt(&tctx, "warn", json!({
                "no_lock_manager": true,
            }));
        }
    }

    // Minimal Facts v1: apply attempt summary (include lock_wait_ms when present)
    emit_apply_attempt(&tctx, "success", json!({
        "lock_wait_ms": lock_wait_ms,
    }));

    // Policy gating: refuse to proceed when preflight would STOP, unless override is set.
    if !api.policy.override_preflight && !dry {
        let mut gating_errors: Vec<String> = Vec::new();
        // Global rescue check
        if api.policy.require_rescue && !crate::rescue::verify_rescue_tools_with_exec_min(api.policy.rescue_exec_check, api.policy.rescue_min_count) {
            gating_errors.push("rescue profile unavailable".to_string());
        }
        for act in &plan.actions {
            match act {
                Action::EnsureSymlink { source, target } => {
                    if let Err(e) = crate::preflight::ensure_mount_rw_exec(std::path::Path::new("/usr")) {
                        gating_errors.push(format!("/usr not rw+exec: {}", e));
                    }
                    if let Err(e) = crate::preflight::ensure_mount_rw_exec(&target.as_path()) {
                        gating_errors.push(format!("target not rw+exec: {} (target={})", e, target.as_path().display()));
                    }
                    if let Err(e) = crate::preflight::check_immutable(&target.as_path()) {
                        gating_errors.push(format!("immutable target: {} (target={})", e, target.as_path().display()));
                    }
                    if let Err(e) = crate::preflight::check_source_trust(&source.as_path(), api.policy.force_untrusted_source) {
                        if api.policy.force_untrusted_source {
                            // allowed as warning in preflight; do not STOP here
                        } else {
                            gating_errors.push(format!("untrusted source: {}", e));
                        }
                    }
                    if api.policy.strict_ownership {
                        match &api.owner {
                            Some(oracle) => {
                                if let Err(e) = oracle.owner_of(target) {
                                    gating_errors.push(format!("strict ownership check failed: {}", e));
                                }
                            }
                            None => {
                                gating_errors.push("strict ownership policy requires OwnershipOracle".to_string());
                            }
                        }
                    }
                    if !api.policy.allow_roots.is_empty() {
                        let target_abs = target.as_path();
                        let in_allowed = api.policy.allow_roots.iter().any(|r| target_abs.starts_with(r));
                        if !in_allowed {
                            gating_errors.push(format!("target outside allowed roots: {}", target_abs.display()));
                        }
                    }
                    if api.policy.forbid_paths.iter().any(|f| target.as_path().starts_with(f)) {
                        gating_errors.push(format!("target in forbidden path: {}", target.as_path().display()));
                    }
                }
                Action::RestoreFromBackup { target } => {
                    if let Err(e) = crate::preflight::ensure_mount_rw_exec(std::path::Path::new("/usr")) {
                        gating_errors.push(format!("/usr not rw+exec: {}", e));
                    }
                    if let Err(e) = crate::preflight::ensure_mount_rw_exec(&target.as_path()) {
                        gating_errors.push(format!("target not rw+exec: {} (target={})", e, target.as_path().display()));
                    }
                    if let Err(e) = crate::preflight::check_immutable(&target.as_path()) {
                        gating_errors.push(format!("immutable target: {} (target={})", e, target.as_path().display()));
                    }
                    if !api.policy.allow_roots.is_empty() {
                        let target_abs = target.as_path();
                        let in_allowed = api.policy.allow_roots.iter().any(|r| target_abs.starts_with(r));
                        if !in_allowed {
                            gating_errors.push(format!("target outside allowed roots: {}", target_abs.display()));
                        }
                    }
                    if api.policy.forbid_paths.iter().any(|f| target.as_path().starts_with(f)) {
                        gating_errors.push(format!("target in forbidden path: {}", target.as_path().display()));
                    }
                }
            }
        }
        if !gating_errors.is_empty() {
            // Emit final failure summary with E_POLICY and exit code
            let ec = exit_code_for(ErrorId::E_POLICY);
            emit_apply_result(&tctx, "failure", json!({
                "error_id": "E_POLICY",
                "exit_code": ec,
            }));
            let duration_ms = t0.elapsed().as_millis() as u64;
            return ApplyReport {
                executed,
                duration_ms,
                errors: gating_errors,
                plan_uuid: Some(pid),
                rolled_back,
                rollback_errors,
            };
        }
    }

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
                let before_kind = kind_of(&target.as_path());
                // Compute before/after hashes
                let before_hash = match resolve_symlink_target(&target.as_path()) {
                    Some(p) => sha256_hex_of(&p),
                    None => sha256_hex_of(&target.as_path()),
                };
                let after_hash = sha256_hex_of(&source.as_path());
                let mut this_err_id: Option<ErrorId> = None;
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
                    Err(e) => {
                        // Map to Silver-tier error ids for atomic swap/exdev
                        let emsg = e.to_string();
                        this_err_id = Some(if emsg.contains("sidecar write failed") {
                            ErrorId::E_POLICY
                        } else {
                            match e.raw_os_error() {
                                Some(code) if code == libc::EXDEV => ErrorId::E_EXDEV,
                                _ => ErrorId::E_ATOMIC_SWAP,
                            }
                        });
                        errors.push(format!(
                            "symlink {} -> {} failed: {}",
                            source.as_path().display(),
                            target.as_path().display(),
                            e
                        ));
                    }
                }
                // Minimal Facts v1: per-action result
                let decision = if errors.is_empty() { "success" } else { "failure" };
                let mut extra = json!({
                    "action_id": _aid.to_string(),
                    "path": target.as_path().display().to_string(),
                    "degraded": if degraded_used { Some(true) } else { None },
                    "duration_ms": fsync_ms,
                    "before_kind": before_kind,
                    "after_kind": if dry { "symlink".to_string() } else { kind_of(&target.as_path()) },
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
                ensure_provenance(&mut extra);
                // Enrich provenance with uid/gid/pkg when oracle is present
                if let Some(oracle) = &api.owner {
                    if let Ok(info) = oracle.owner_of(target) {
                        if let Some(obj) = extra.as_object_mut() {
                            let prov = obj
                                .entry("provenance").or_insert(json!({}))
                                .as_object_mut()
                                .unwrap();
                            prov.insert("uid".to_string(), json!(info.uid));
                            prov.insert("gid".to_string(), json!(info.gid));
                            prov.insert("pkg".to_string(), json!(info.pkg));
                        }
                    }
                }
                if errors.is_empty() && fsync_ms > FSYNC_WARN_MS {
                    let obj = extra.as_object_mut().unwrap();
                    obj.insert("severity".to_string(), json!("warn"));
                }
                if decision == "failure" {
                    let obj = extra.as_object_mut().unwrap();
                    if let Some(id) = this_err_id {
                        obj.insert("error_id".to_string(), json!(crate::api::errors::id_str(id)));
                        obj.insert("exit_code".to_string(), json!(exit_code_for(id)));
                    } else {
                        obj.insert("error_id".to_string(), json!("E_GENERIC"));
                        obj.insert("exit_code".to_string(), json!(1));
                    }
                }
                emit_apply_result(&tctx, decision, extra);
            }
            Action::RestoreFromBackup { target } => {
                // Minimal Facts v1: per-action attempt
                emit_apply_attempt(&tctx, "success", json!({
                    "action_id": _aid.to_string(),
                    "path": target.as_path().display().to_string(),
                }));
                let mut this_err_id: Option<ErrorId> = None;
                let before_kind = kind_of(&target.as_path());
                match fs::restore_file(
                    &target.as_path(),
                    dry,
                    api.policy.force_restore_best_effort,
                    &api.policy.backup_tag,
                ) {
                    Ok(()) => executed.push(act.clone()),
                    Err(e) => {
                        // Map to Silver-tier error ids
                        use std::io::ErrorKind;
                        this_err_id = Some(match e.kind() {
                            ErrorKind::NotFound => ErrorId::E_BACKUP_MISSING,
                            _ => ErrorId::E_RESTORE_FAILED,
                        });
                        errors.push(format!(
                            "restore {} failed: {}",
                            target.as_path().display(),
                            e
                        ));
                    }
                }
                // Minimal Facts v1: per-action result
                let decision = if errors.is_empty() { "success" } else { "failure" };
                let mut extra = json!({
                    "action_id": _aid.to_string(),
                    "path": target.as_path().display().to_string(),
                    "before_kind": before_kind,
                    "after_kind": if dry { before_kind } else { kind_of(&target.as_path()) },
                });
                ensure_provenance(&mut extra);
                // Enrich provenance with uid/gid/pkg when oracle is present
                if let Some(oracle) = &api.owner {
                    if let Ok(info) = oracle.owner_of(target) {
                        if let Some(obj) = extra.as_object_mut() {
                            let prov = obj
                                .entry("provenance").or_insert(json!({}))
                                .as_object_mut()
                                .unwrap();
                            prov.insert("uid".to_string(), json!(info.uid));
                            prov.insert("gid".to_string(), json!(info.gid));
                            prov.insert("pkg".to_string(), json!(info.pkg));
                        }
                    }
                }
                if decision == "failure" {
                    let obj = extra.as_object_mut().unwrap();
                    if let Some(id) = this_err_id {
                        obj.insert("error_id".to_string(), json!(crate::api::errors::id_str(id)));
                        obj.insert("exit_code".to_string(), json!(exit_code_for(id)));
                    } else {
                        obj.insert("error_id".to_string(), json!("E_GENERIC"));
                        obj.insert("exit_code".to_string(), json!(1));
                    }
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
        } else {
            // H3: Missing smoke runner when required
            if api.policy.require_smoke_in_commit {
                errors.push("smoke runner missing".to_string());
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
            // Construct a minimal attestation bundle with plan_id and executed actions count
            let bundle_json = json!({
                "plan_id": pid.to_string(),
                "executed": executed.len(),
                "rolled_back": rolled_back,
            });
            let bundle: Vec<u8> = serde_json::to_vec(&bundle_json).unwrap_or_default();
            if let Ok(sig) = att.sign(&bundle) {
                let sig_b64 = base64::engine::general_purpose::STANDARD.encode(sig.0.clone());
                // Compute sha256 of bundle for bundle_hash
                let mut hasher = sha2::Sha256::new();
                use sha2::Digest as _;
                hasher.update(&bundle);
                let bundle_hash = hex::encode(hasher.finalize());
                let att_json = json!({
                    "sig_alg": att.algorithm(),
                    "signature": sig_b64,
                    "bundle_hash": bundle_hash,
                    "public_key_id": att.key_id(),
                });
                // Merge attestation into fields
                let obj = fields.as_object_mut().unwrap();
                obj.insert("attestation".to_string(), att_json);
            }
        }
    }
    // we already include ts/stage in helper
    // If we failed post-apply due to smoke, emit E_SMOKE at summary level
    if decision == "failure" {
        if errors.iter().any(|e| e.contains("smoke")) {
            if let Some(obj) = fields.as_object_mut() {
                obj.insert("error_id".to_string(), json!(crate::api::errors::id_str(ErrorId::E_SMOKE)));
                obj.insert("exit_code".to_string(), json!(exit_code_for(ErrorId::E_SMOKE)));
            }
        }
    }
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
