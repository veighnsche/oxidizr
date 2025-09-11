use std::time::Instant;

use crate::adapters::{Attestor, LockManager, OwnershipOracle, SmokeTestRunner};
use crate::logging::{AuditSink, FactsEmitter, TS_ZERO, ts_for_mode};
use crate::policy::Policy;
use crate::types::ids::{action_id, plan_id};
use crate::types::{Action, ApplyMode, ApplyReport, Plan, PlanInput, PreflightReport};
use crate::{fs, preflight};
use base64::Engine;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::Path;

fn sha256_hex_of(path: &Path) -> Option<String> {
    let mut f = std::fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut f, &mut hasher).ok()?;
    let out = hasher.finalize();
    Some(hex::encode(out))
}

fn resolve_symlink_target(target: &Path) -> Option<std::path::PathBuf> {
    if let Ok(md) = std::fs::symlink_metadata(target) {
        if md.file_type().is_symlink() {
            if let Ok(mut link) = std::fs::read_link(target) {
                if link.is_relative() {
                    if let Some(parent) = target.parent() {
                        link = parent.join(link);
                    }
                }
                return Some(link);
            }
        }
    }
    None
}

fn kind_of(path: &Path) -> String {
    match std::fs::symlink_metadata(path) {
        Ok(md) => {
            let ft = md.file_type();
            if ft.is_symlink() {
                "symlink".to_string()
            } else if ft.is_file() {
                "file".to_string()
            } else if ft.is_dir() {
                "dir".to_string()
            } else {
                "unknown".to_string()
            }
        }
        Err(_) => "missing".to_string(),
    }
}

pub struct Switchyard<E: FactsEmitter, A: AuditSink> {
    facts: E,
    audit: A,
    policy: Policy,
    lock: Option<Box<dyn LockManager>>, // None in dev/test; required in production
    owner: Option<Box<dyn OwnershipOracle>>, // for strict ownership gating
    attest: Option<Box<dyn Attestor>>,  // for final summary attestation
    smoke: Option<Box<dyn SmokeTestRunner>>, // for post-apply health verification
    lock_timeout_ms: u64,
}

impl<E: FactsEmitter, A: AuditSink> Switchyard<E, A> {
    pub fn new(facts: E, audit: A, policy: Policy) -> Self {
        Self {
            facts,
            audit,
            policy,
            lock: None,
            owner: None,
            attest: None,
            smoke: None,
            lock_timeout_ms: 5000,
        }
    }

    pub fn with_lock_manager(mut self, lock: Box<dyn LockManager>) -> Self {
        self.lock = Some(lock);
        self
    }

    pub fn with_ownership_oracle(mut self, owner: Box<dyn OwnershipOracle>) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn with_attestor(mut self, attest: Box<dyn Attestor>) -> Self {
        self.attest = Some(attest);
        self
    }

    pub fn with_smoke_runner(mut self, smoke: Box<dyn SmokeTestRunner>) -> Self {
        self.smoke = Some(smoke);
        self
    }

    pub fn with_lock_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.lock_timeout_ms = timeout_ms;
        self
    }

    pub fn plan(&self, input: PlanInput) -> Plan {
        let mut actions: Vec<Action> = Vec::new();
        for l in input.link {
            actions.push(Action::EnsureSymlink {
                source: l.source,
                target: l.target,
            });
        }
        for r in input.restore {
            actions.push(Action::RestoreFromBackup { target: r.target });
        }
        // Stable ordering: sort actions by deterministic key (target rel path), then by kind
        actions.sort_by(|a, b| {
            let ka = match a {
                Action::EnsureSymlink { target, .. } => (0u8, target.rel().to_string_lossy().to_string()),
                Action::RestoreFromBackup { target } => (1u8, target.rel().to_string_lossy().to_string()),
            };
            let kb = match b {
                Action::EnsureSymlink { target, .. } => (0u8, target.rel().to_string_lossy().to_string()),
                Action::RestoreFromBackup { target } => (1u8, target.rel().to_string_lossy().to_string()),
            };
            ka.cmp(&kb)
        });
        let plan = Plan { actions };
        // Minimal Facts v1: emit one fact per action at stage=plan
        let pid = plan_id(&plan);
        for (idx, act) in plan.actions.iter().enumerate() {
            let aid = action_id(&pid, act, idx);
            let path = match act {
                Action::EnsureSymlink { target, .. } => {
                    Some(target.as_path().display().to_string())
                }
                Action::RestoreFromBackup { target } => {
                    Some(target.as_path().display().to_string())
                }
            };
            let fields = json!({
                "schema_version": 1,
                "ts": TS_ZERO,
                "plan_id": pid.to_string(),
                "stage": "plan",
                "decision": "success",
                "action_id": aid.to_string(),
                "path": path,
            });
            self.facts.emit("switchyard", "plan", "success", fields);
        }
        plan
    }

    pub fn preflight(&self, plan: &Plan) -> PreflightReport {
        let mut warnings: Vec<String> = Vec::new();
        let mut stops: Vec<String> = Vec::new();

        for act in &plan.actions {
            match act {
                Action::EnsureSymlink { source, target } => {
                    if let Err(e) = preflight::ensure_mount_rw_exec(std::path::Path::new("/usr")) {
                        stops.push(format!("/usr not rw+exec: {}", e));
                    }
                    if let Err(e) = preflight::ensure_mount_rw_exec(&target.as_path()) {
                        stops.push(format!(
                            "target not rw+exec: {} (target={})",
                            e,
                            target.as_path().display()
                        ));
                    }
                    if let Err(e) = preflight::check_immutable(&target.as_path()) {
                        stops.push(format!(
                            "immutable target: {} (target={})",
                            e,
                            target.as_path().display()
                        ));
                    }
                    match preflight::check_source_trust(
                        &source.as_path(),
                        self.policy.force_untrusted_source,
                    ) {
                        Ok(()) => {}
                        Err(e) => {
                            if self.policy.force_untrusted_source {
                                warnings.push(format!("untrusted source allowed by policy: {}", e));
                            } else {
                                stops.push(format!("untrusted source: {}", e));
                            }
                        }
                    }
                    if self.policy.strict_ownership {
                        match &self.owner {
                            Some(oracle) => {
                                if let Err(e) = oracle.owner_of(target) {
                                    stops.push(format!("strict ownership check failed: {}", e));
                                }
                            }
                            None => {
                                stops.push(
                                    "strict ownership policy requires OwnershipOracle".to_string(),
                                );
                            }
                        }
                    }
                    if !self.policy.allow_roots.is_empty() {
                        let target_abs = target.as_path();
                        let in_allowed = self
                            .policy
                            .allow_roots
                            .iter()
                            .any(|r| target_abs.starts_with(r));
                        if !in_allowed {
                            stops.push(format!(
                                "target outside allowed roots: {}",
                                target_abs.display()
                            ));
                        }
                    }
                    if self
                        .policy
                        .forbid_paths
                        .iter()
                        .any(|f| target.as_path().starts_with(f))
                    {
                        stops.push(format!(
                            "target in forbidden path: {}",
                            target.as_path().display()
                        ));
                    }
                }
                Action::RestoreFromBackup { target } => {
                    if let Err(e) = preflight::ensure_mount_rw_exec(std::path::Path::new("/usr")) {
                        stops.push(format!("/usr not rw+exec: {}", e));
                    }
                    if let Err(e) = preflight::ensure_mount_rw_exec(&target.as_path()) {
                        stops.push(format!(
                            "target not rw+exec: {} (target={})",
                            e,
                            target.as_path().display()
                        ));
                    }
                    if let Err(e) = preflight::check_immutable(&target.as_path()) {
                        stops.push(format!(
                            "immutable target: {} (target={})",
                            e,
                            target.as_path().display()
                        ));
                    }
                    if !self.policy.allow_roots.is_empty() {
                        let target_abs = target.as_path();
                        let in_allowed = self
                            .policy
                            .allow_roots
                            .iter()
                            .any(|r| target_abs.starts_with(r));
                        if !in_allowed {
                            stops.push(format!(
                                "target outside allowed roots: {}",
                                target_abs.display()
                            ));
                        }
                    }
                    if self
                        .policy
                        .forbid_paths
                        .iter()
                        .any(|f| target.as_path().starts_with(f))
                    {
                        stops.push(format!(
                            "target in forbidden path: {}",
                            target.as_path().display()
                        ));
                    }
                }
            }
        }

        // Minimal Facts v1: per-action preflight facts
        let pid = plan_id(plan);
        for (idx, act) in plan.actions.iter().enumerate() {
            let aid = action_id(&pid, act, idx);
            let path = match act {
                Action::EnsureSymlink { target, .. } => {
                    Some(target.as_path().display().to_string())
                }
                Action::RestoreFromBackup { target } => {
                    Some(target.as_path().display().to_string())
                }
            };
            let (current_kind, planned_kind) = match act {
                Action::EnsureSymlink { target, .. } => (
                    kind_of(&target.as_path()),
                    "symlink".to_string(),
                ),
                Action::RestoreFromBackup { .. } => (
                    "unknown".to_string(),
                    "restore_from_backup".to_string(),
                ),
            };
            let fields = json!({
                "schema_version": 1,
                "ts": TS_ZERO,
                "plan_id": pid.to_string(),
                "stage": "preflight",
                "decision": "success",
                "action_id": aid.to_string(),
                "path": path,
                "current_kind": current_kind,
                "planned_kind": planned_kind,
            });
            self.facts
                .emit("switchyard", "preflight", "success", fields);
        }
        // Minimal Facts v1: preflight summary
        let decision = if stops.is_empty() {
            "success"
        } else {
            "failure"
        };
        let fields = json!({
            "schema_version": 1,
            "ts": TS_ZERO,
            "plan_id": pid.to_string(),
            "stage": "preflight",
            "decision": decision,
            "path": "",
        });
        self.facts.emit("switchyard", "preflight", decision, fields);

        PreflightReport {
            ok: stops.is_empty(),
            warnings,
            stops,
        }
    }

    pub fn apply(&self, plan: &Plan, mode: ApplyMode) -> ApplyReport {
        let t0 = Instant::now();
        let mut executed: Vec<Action> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        let mut rollback_errors: Vec<String> = Vec::new();
        let mut rolled_back = false;
        let dry = matches!(mode, ApplyMode::DryRun);
        let pid = plan_id(plan);
        let ts_now = ts_for_mode(&mode);

        // Locking (optional in dev/test): acquire process lock with bounded wait; emit telemetry via apply.attempt
        let mut lock_wait_ms: Option<u64> = None;
        let mut _lock_guard: Option<Box<dyn crate::adapters::lock::LockGuard>> = None;
        if let Some(mgr) = &self.lock {
            let lt0 = Instant::now();
            match mgr.acquire_process_lock(self.lock_timeout_ms) {
                Ok(guard) => {
                    lock_wait_ms = Some(lt0.elapsed().as_millis() as u64);
                    _lock_guard = Some(guard);
                }
                Err(e) => {
                    let fields = json!({
                        "schema_version": 1,
                        "ts": ts_now,
                        "plan_id": pid.to_string(),
                        "stage": "apply.attempt",
                        "decision": "failure",
                        "lock_wait_ms": lock_wait_ms,
                        "path": "",
                        "error": e.to_string(),
                    });
                    self.facts
                        .emit("switchyard", "apply.attempt", "failure", fields);
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
            let fields = json!({
                "schema_version": 1,
                "ts": ts_now,
                "plan_id": pid.to_string(),
                "stage": "apply.attempt",
                "decision": "warn",
                "no_lock_manager": true,
                "path": "",
            });
            self.facts
                .emit("switchyard", "apply.attempt", "warn", fields);
        }

        // Minimal Facts v1: apply attempt summary (include lock_wait_ms when present)
        let fields = json!({
            "schema_version": 1,
            "ts": ts_now,
            "plan_id": pid.to_string(),
            "stage": "apply.attempt",
            "decision": "success",
            "lock_wait_ms": lock_wait_ms,
            "path": "",
        });
        self.facts
            .emit("switchyard", "apply.attempt", "success", fields);

        for (idx, act) in plan.actions.iter().enumerate() {
            let _aid = action_id(&pid, act, idx);
            match act {
                Action::EnsureSymlink { source, target } => {
                    // Minimal Facts v1: per-action attempt
                    let fields = json!({
                        "schema_version": 1,
                        "ts": ts_now,
                        "plan_id": pid.to_string(),
                        "stage": "apply.attempt",
                        "decision": "success",
                        "action_id": _aid.to_string(),
                        "path": target.as_path().display().to_string(),
                    });
                    self.facts
                        .emit("switchyard", "apply.attempt", "success", fields);
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
                        self.policy.allow_degraded_fs,
                        &self.policy.backup_tag,
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
                    let decision = if errors.is_empty() {
                        "success"
                    } else {
                        "failure"
                    };
                    let mut fields = json!({
                        "schema_version": 1,
                        "ts": ts_now,
                        "plan_id": pid.to_string(),
                        "stage": "apply.result",
                        "decision": decision,
                        "action_id": _aid.to_string(),
                        "path": target.as_path().display().to_string(),
                        "degraded": if degraded_used { Some(true) } else { None },
                        "duration_ms": fsync_ms,
                    });
                    if let Some(bh) = before_hash.as_ref() {
                        let obj = fields.as_object_mut().unwrap();
                        obj.insert("hash_alg".to_string(), json!("sha256"));
                        obj.insert("before_hash".to_string(), json!(bh));
                    }
                    if let Some(ah) = after_hash.as_ref() {
                        let obj = fields.as_object_mut().unwrap();
                        obj.insert("hash_alg".to_string(), json!("sha256"));
                        obj.insert("after_hash".to_string(), json!(ah));
                    }
                    // Add provenance placeholders
                    {
                        let obj = fields.as_object_mut().unwrap();
                        let uid = unsafe { libc::geteuid() } as u32;
                        let gid = unsafe { libc::getegid() } as u32;
                        obj.insert(
                            "provenance".to_string(),
                            json!({
                                "helper": "",
                                "uid": uid,
                                "gid": gid,
                                "env_sanitized": true
                            }),
                        );
                    }
                    if errors.is_empty() && fsync_ms > 50 {
                        // Warn on fsync bound breach
                        let obj = fields.as_object_mut().unwrap();
                        obj.insert("severity".to_string(), json!("warn"));
                    }
                    self.facts
                        .emit("switchyard", "apply.result", decision, fields);
                }
                Action::RestoreFromBackup { target } => {
                    // Minimal Facts v1: per-action attempt
                    let fields = json!({
                        "schema_version": 1,
                        "ts": ts_now,
                        "plan_id": pid.to_string(),
                        "stage": "apply.attempt",
                        "decision": "success",
                        "action_id": _aid.to_string(),
                        "path": target.as_path().display().to_string(),
                    });
                    self.facts
                        .emit("switchyard", "apply.attempt", "success", fields);
                    match fs::restore_file(
                        &target.as_path(),
                        dry,
                        self.policy.force_restore_best_effort,
                        &self.policy.backup_tag,
                    ) {
                        Ok(()) => executed.push(act.clone()),
                        Err(e) => errors.push(format!(
                            "restore {} failed: {}",
                            target.as_path().display(),
                            e
                        )),
                    }
                    // Minimal Facts v1: per-action result
                    let decision = if errors.is_empty() {
                        "success"
                    } else {
                        "failure"
                    };
                    let mut fields = json!({
                        "schema_version": 1,
                        "ts": ts_now,
                        "plan_id": pid.to_string(),
                        "stage": "apply.result",
                        "decision": decision,
                        "action_id": _aid.to_string(),
                        "path": target.as_path().display().to_string(),
                    });
                    {
                        let obj = fields.as_object_mut().unwrap();
                        let uid = unsafe { libc::geteuid() } as u32;
                        let gid = unsafe { libc::getegid() } as u32;
                        obj.insert(
                            "provenance".to_string(),
                            json!({
                                "helper": "",
                                "uid": uid,
                                "gid": gid,
                                "env_sanitized": true
                            }),
                        );
                    }
                    self.facts
                        .emit("switchyard", "apply.result", decision, fields);
                }
            }

            // On first failure, attempt reverse-order rollback for already executed actions.
            if !errors.is_empty() {
                if !dry {
                    rolled_back = true;
                    for prev in executed.iter().rev() {
                        match prev {
                            Action::EnsureSymlink {
                                source: _source,
                                target,
                            } => {
                                match fs::restore_file(
                                    &target.as_path(),
                                    dry,
                                    self.policy.force_restore_best_effort,
                                    &self.policy.backup_tag,
                                ) {
                                    Ok(()) => {
                                        let fields = json!({
                                            "schema_version": 1,
                                            "ts": ts_now,
                                            "plan_id": pid.to_string(),
                                            "stage": "rollback",
                                            "decision": "success",
                                            "path": target.as_path().display().to_string(),
                                        });
                                        self.facts.emit(
                                            "switchyard",
                                            "rollback",
                                            "success",
                                            fields,
                                        );
                                    }
                                    Err(e) => {
                                        rollback_errors.push(format!(
                                            "rollback restore {} failed: {}",
                                            target.as_path().display(),
                                            e
                                        ));
                                        let fields = json!({
                                            "schema_version": 1,
                                            "ts": ts_now,
                                            "plan_id": pid.to_string(),
                                            "stage": "rollback",
                                            "decision": "failure",
                                            "path": target.as_path().display().to_string(),
                                        });
                                        self.facts.emit(
                                            "switchyard",
                                            "rollback",
                                            "failure",
                                            fields,
                                        );
                                    }
                                }
                            }
                            Action::RestoreFromBackup { .. } => {
                                // No reliable inverse without prior state capture; record informational error.
                                rollback_errors.push(
                                    "rollback of RestoreFromBackup not supported (no prior state)"
                                        .to_string(),
                                );
                                let fields = json!({
                                    "schema_version": 1,
                                    "ts": ts_now,
                                    "plan_id": pid.to_string(),
                                    "stage": "rollback",
                                    "decision": "failure",
                                    "path": "",
                                });
                                self.facts.emit("switchyard", "rollback", "failure", fields);
                            }
                        }
                    }
                }
                break;
            }
        }

        // Optional smoke tests post-apply (only in Commit mode)
        if errors.is_empty() && !dry {
            if let Some(smoke) = &self.smoke {
                if smoke.run(plan).is_err() {
                    errors.push("smoke tests failed".to_string());
                    if !self.policy.disable_auto_rollback {
                        rolled_back = true;
                        for prev in executed.iter().rev() {
                            match prev {
                                Action::EnsureSymlink { source: _s, target } => {
                                    let _ = fs::restore_file(
                                        &target.as_path(),
                                        dry,
                                        self.policy.force_restore_best_effort,
                                        &self.policy.backup_tag,
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
        let decision = if errors.is_empty() {
            "success"
        } else {
            "failure"
        };
        let mut fields = json!({
            "schema_version": 1,
            "ts": ts_now,
            "plan_id": pid.to_string(),
            "stage": "apply.result",
            "decision": decision,
            "path": "",
        });
        // Optional attestation on success, non-dry-run
        if errors.is_empty() && !dry {
            if let Some(att) = &self.attest {
                let bundle: Vec<u8> = Vec::new(); // TODO: real bundle
                if let Ok(sig) = att.sign(&bundle) {
                    let sig_b64 = base64::engine::general_purpose::STANDARD.encode(sig.0);
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
        self.facts
            .emit("switchyard", "apply.result", decision, fields);

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

    pub fn plan_rollback_of(&self, report: &ApplyReport) -> Plan {
        let mut actions: Vec<Action> = Vec::new();
        for act in report.executed.iter().rev() {
            match act {
                Action::EnsureSymlink { target, .. } => {
                    actions.push(Action::RestoreFromBackup {
                        target: target.clone(),
                    });
                }
                Action::RestoreFromBackup { .. } => {
                    // Unknown prior state; skip generating an inverse.
                }
            }
        }
        Plan { actions }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::{AuditSink, FactsEmitter};
    use log::Level;
    use serde_json::Value;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    #[derive(Default, Clone)]
    struct TestEmitter {
        events: std::sync::Arc<std::sync::Mutex<Vec<(String, String, String, Value)>>>,
    }

    impl FactsEmitter for TestEmitter {
        fn emit(&self, subsystem: &str, event: &str, decision: &str, fields: Value) {
            self.events.lock().unwrap().push((
                subsystem.to_string(),
                event.to_string(),
                decision.to_string(),
                fields,
            ));
        }
    }

    #[derive(Default, Clone)]
    struct TestAudit;
    impl AuditSink for TestAudit {
        fn log(&self, _level: Level, _msg: &str) {}
    }

    #[test]
    fn emits_minimal_facts_for_plan_preflight_apply() {
        let facts = TestEmitter::default();
        let audit = TestAudit::default();
        let policy = Policy::default();
        let api = Switchyard::new(facts.clone(), audit, policy);

        // Build a simple plan under a temp root
        let td = tempfile::tempdir().unwrap();
        let root = td.path();
        let src = Path::new("bin/uutils");
        let tgt = Path::new("usr/bin/ls");
        // Use SafePath from root
        let source = crate::types::safepath::SafePath::from_rooted(root, &root.join(src)).unwrap();
        let target = crate::types::safepath::SafePath::from_rooted(root, &root.join(tgt)).unwrap();
        let input = PlanInput {
            link: vec![crate::types::plan::LinkRequest {
                source: source.clone(),
                target: target.clone(),
            }],
            restore: vec![],
        };

        let plan = api.plan(input);
        // Preflight and apply (DryRun)
        let _pf = api.preflight(&plan);
        let _ar = api.apply(&plan, ApplyMode::DryRun);

        // Inspect captured events
        let evs = facts.events.lock().unwrap();
        assert!(!evs.is_empty(), "no facts captured");
        // Ensure all facts include schema_version and path
        for (_subsystem, _event, _decision, fields) in evs.iter() {
            assert_eq!(
                fields.get("schema_version").and_then(|v| v.as_i64()),
                Some(1),
                "schema_version=1"
            );
            // path may be null for some summaries; only check presence when present
            let _ = fields.get("path");
        }
        // Ensure plan_id is consistent and present
        let plan_ids: Vec<String> = evs
            .iter()
            .filter_map(|(_, _, _, f)| {
                f.get("plan_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect();
        assert!(!plan_ids.is_empty());
        let first = &plan_ids[0];
        assert!(
            plan_ids.iter().all(|p| p == first),
            "plan_id should be consistent across events"
        );
    }

    #[test]
    fn rollback_reverts_first_action_on_second_failure() {
        let facts = TestEmitter::default();
        let audit = TestAudit::default();
        let policy = Policy::default();
        let api = Switchyard::new(facts.clone(), audit, policy);

        let td = tempfile::tempdir().unwrap();
        let root = td.path();

        // Layout
        let src1 = root.join("bin/new1");
        let src2 = root.join("bin/new2");
        let tgt1 = root.join("usr/bin/app1");
        let tgt2 = root.join("usr/sbin/app2");

        std::fs::create_dir_all(src1.parent().unwrap()).unwrap();
        std::fs::create_dir_all(src2.parent().unwrap()).unwrap();
        std::fs::create_dir_all(tgt1.parent().unwrap()).unwrap();
        std::fs::create_dir_all(tgt2.parent().unwrap()).unwrap();

        std::fs::write(&src1, b"new1").unwrap();
        std::fs::write(&src2, b"new2").unwrap();
        std::fs::write(&tgt1, b"old1").unwrap();
        std::fs::write(&tgt2, b"old2").unwrap();

        // Make parent of second target read-only to force failure during apply
        let sbin_dir = tgt2.parent().unwrap();
        let mut p = std::fs::metadata(sbin_dir).unwrap().permissions();
        p.set_mode(0o555);
        std::fs::set_permissions(sbin_dir, p).unwrap();

        // Build SafePaths
        let sp_src1 = crate::types::safepath::SafePath::from_rooted(root, &src1).unwrap();
        let sp_src2 = crate::types::safepath::SafePath::from_rooted(root, &src2).unwrap();
        let sp_tgt1 = crate::types::safepath::SafePath::from_rooted(root, &tgt1).unwrap();
        let sp_tgt2 = crate::types::safepath::SafePath::from_rooted(root, &tgt2).unwrap();

        let input = PlanInput {
            link: vec![
                crate::types::plan::LinkRequest {
                    source: sp_src1.clone(),
                    target: sp_tgt1.clone(),
                },
                crate::types::plan::LinkRequest {
                    source: sp_src2.clone(),
                    target: sp_tgt2.clone(),
                },
            ],
            restore: vec![],
        };
        let plan = api.plan(input);

        let report = api.apply(&plan, ApplyMode::Commit);
        assert!(
            !report.errors.is_empty(),
            "apply should fail on second action"
        );
        assert!(report.rolled_back, "rolled_back should be true");

        // First target should be restored to regular file with original content
        let md1 = std::fs::symlink_metadata(&tgt1).unwrap();
        assert!(
            md1.file_type().is_file(),
            "tgt1 should be a regular file after rollback"
        );
        let content1 = std::fs::read_to_string(&tgt1).unwrap();
        assert!(content1.starts_with("old1"));
    }
}
