/// Placeholder

use std::time::Instant;

use crate::{preflight, fs};
use crate::logging::{FactsEmitter, AuditSink};
use crate::policy::Policy;
use crate::types::{PlanInput, Plan, Action, PreflightReport, ApplyMode, ApplyReport};
use crate::types::ids::{plan_id, action_id};
use serde_json::json;

pub struct Switchyard<E: FactsEmitter, A: AuditSink> {
    facts: E,
    audit: A,
    policy: Policy,
}

impl<E: FactsEmitter, A: AuditSink> Switchyard<E, A> {
    pub fn new(facts: E, audit: A, policy: Policy) -> Self { Self { facts, audit, policy } }

    pub fn plan(&self, input: PlanInput) -> Plan {
        let mut actions: Vec<Action> = Vec::new();
        for l in input.link {
            actions.push(Action::EnsureSymlink { source: l.source, target: l.target });
        }
        for r in input.restore {
            actions.push(Action::RestoreFromBackup { target: r.target });
        }
        let plan = Plan { actions };
        // Minimal Facts v1: emit one fact per action at stage=plan
        let pid = plan_id(&plan);
        for (idx, act) in plan.actions.iter().enumerate() {
            let aid = action_id(&pid, act, idx);
            let path = match act {
                Action::EnsureSymlink { target, .. } => Some(target.as_path().display().to_string()),
                Action::RestoreFromBackup { target } => Some(target.as_path().display().to_string()),
            };
            let fields = json!({
                "schema_version": 1,
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
                        stops.push(format!("target not rw+exec: {} (target={})", e, target.as_path().display()));
                    }
                    if let Err(e) = preflight::check_immutable(&target.as_path()) {
                        stops.push(format!("immutable target: {} (target={})", e, target.as_path().display()));
                    }
                    match preflight::check_source_trust(&source.as_path(), self.policy.force_untrusted_source) {
                        Ok(()) => {}
                        Err(e) => {
                            if self.policy.force_untrusted_source {
                                warnings.push(format!("untrusted source allowed by policy: {}", e));
                            } else {
                                stops.push(format!("untrusted source: {}", e));
                            }
                        }
                    }
                    if !self.policy.allow_roots.is_empty() {
                        let target_abs = target.as_path();
                        let in_allowed = self.policy.allow_roots.iter().any(|r| target_abs.starts_with(r));
                        if !in_allowed {
                            stops.push(format!("target outside allowed roots: {}", target_abs.display()));
                        }
                    }
                    if self.policy.forbid_paths.iter().any(|f| target.as_path().starts_with(f)) {
                        stops.push(format!("target in forbidden path: {}", target.as_path().display()));
                    }
                }
                Action::RestoreFromBackup { target } => {
                    if let Err(e) = preflight::ensure_mount_rw_exec(std::path::Path::new("/usr")) {
                        stops.push(format!("/usr not rw+exec: {}", e));
                    }
                    if let Err(e) = preflight::ensure_mount_rw_exec(&target.as_path()) {
                        stops.push(format!("target not rw+exec: {} (target={})", e, target.as_path().display()));
                    }
                    if let Err(e) = preflight::check_immutable(&target.as_path()) {
                        stops.push(format!("immutable target: {} (target={})", e, target.as_path().display()));
                    }
                    if !self.policy.allow_roots.is_empty() {
                        let target_abs = target.as_path();
                        let in_allowed = self.policy.allow_roots.iter().any(|r| target_abs.starts_with(r));
                        if !in_allowed {
                            stops.push(format!("target outside allowed roots: {}", target_abs.display()));
                        }
                    }
                    if self.policy.forbid_paths.iter().any(|f| target.as_path().starts_with(f)) {
                        stops.push(format!("target in forbidden path: {}", target.as_path().display()));
                    }
                }
            }
        }

        // Minimal Facts v1: per-action preflight facts
        let pid = plan_id(plan);
        for (idx, act) in plan.actions.iter().enumerate() {
            let aid = action_id(&pid, act, idx);
            let path = match act {
                Action::EnsureSymlink { target, .. } => Some(target.as_path().display().to_string()),
                Action::RestoreFromBackup { target } => Some(target.as_path().display().to_string()),
            };
            let fields = json!({
                "schema_version": 1,
                "plan_id": pid.to_string(),
                "stage": "preflight",
                "decision": "success",
                "action_id": aid.to_string(),
                "path": path,
            });
            self.facts.emit("switchyard", "preflight", "success", fields);
        }
        // Minimal Facts v1: preflight summary
        let decision = if stops.is_empty() { "success" } else { "failure" };
        let fields = json!({
            "schema_version": 1,
            "plan_id": pid.to_string(),
            "stage": "preflight",
            "decision": decision,
        });
        self.facts.emit("switchyard", "preflight", decision, fields);

        PreflightReport { ok: stops.is_empty(), warnings, stops }
    }

    pub fn apply(&self, plan: &Plan, mode: ApplyMode) -> ApplyReport {
        let t0 = Instant::now();
        let mut executed: Vec<Action> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        let mut rollback_errors: Vec<String> = Vec::new();
        let mut rolled_back = false;
        let dry = matches!(mode, ApplyMode::DryRun);
        let pid = plan_id(plan);

        // Minimal Facts v1: apply attempt summary
        let fields = json!({
            "schema_version": 1,
            "plan_id": pid.to_string(),
            "stage": "apply.attempt",
            "decision": "success",
        });
        self.facts.emit("switchyard", "apply.attempt", "success", fields);

        for (idx, act) in plan.actions.iter().enumerate() {
            let _aid = action_id(&pid, act, idx);
            match act {
                Action::EnsureSymlink { source, target } => {
                    // Minimal Facts v1: per-action attempt
                    let fields = json!({
                        "schema_version": 1,
                        "plan_id": pid.to_string(),
                        "stage": "apply.attempt",
                        "decision": "success",
                        "action_id": _aid.to_string(),
                        "path": target.as_path().display().to_string(),
                    });
                    self.facts.emit("switchyard", "apply.attempt", "success", fields);
                    match fs::replace_file_with_symlink(&source.as_path(), &target.as_path(), dry) {
                        Ok(()) => executed.push(act.clone()),
                        Err(e) => errors.push(format!(
                            "symlink {} -> {} failed: {}",
                            source.as_path().display(),
                            target.as_path().display(),
                            e
                        )),
                    }
                    // Minimal Facts v1: per-action result
                    let decision = if errors.is_empty() { "success" } else { "failure" };
                    let fields = json!({
                        "schema_version": 1,
                        "plan_id": pid.to_string(),
                        "stage": "apply.result",
                        "decision": decision,
                        "action_id": _aid.to_string(),
                        "path": target.as_path().display().to_string(),
                    });
                    self.facts.emit("switchyard", "apply.result", decision, fields);
                }
                Action::RestoreFromBackup { target } => {
                    // Minimal Facts v1: per-action attempt
                    let fields = json!({
                        "schema_version": 1,
                        "plan_id": pid.to_string(),
                        "stage": "apply.attempt",
                        "decision": "success",
                        "action_id": _aid.to_string(),
                        "path": target.as_path().display().to_string(),
                    });
                    self.facts.emit("switchyard", "apply.attempt", "success", fields);
                    match fs::restore_file(&target.as_path(), dry, self.policy.force_restore_best_effort) {
                        Ok(()) => executed.push(act.clone()),
                        Err(e) => errors.push(format!(
                            "restore {} failed: {}",
                            target.as_path().display(),
                            e
                        )),
                    }
                    // Minimal Facts v1: per-action result
                    let decision = if errors.is_empty() { "success" } else { "failure" };
                    let fields = json!({
                        "schema_version": 1,
                        "plan_id": pid.to_string(),
                        "stage": "apply.result",
                        "decision": decision,
                        "action_id": _aid.to_string(),
                        "path": target.as_path().display().to_string(),
                    });
                    self.facts.emit("switchyard", "apply.result", decision, fields);
                }
            }

            // On first failure, attempt reverse-order rollback for already executed actions.
            if !errors.is_empty() {
                if !dry {
                    rolled_back = true;
                    for prev in executed.iter().rev() {
                        match prev {
                            Action::EnsureSymlink { source: _source, target } => {
                                if let Err(e) = fs::restore_file(&target.as_path(), dry, self.policy.force_restore_best_effort) {
                                    rollback_errors.push(format!(
                                        "rollback restore {} failed: {}",
                                        target.as_path().display(),
                                        e
                                    ));
                                }
                            }
                            Action::RestoreFromBackup { .. } => {
                                // No reliable inverse without prior state capture; record informational error.
                                rollback_errors.push("rollback of RestoreFromBackup not supported (no prior state)".to_string());
                            }
                        }
                    }
                }
                break;
            }
        }

        // Minimal Facts v1: final apply.result summary
        let decision = if errors.is_empty() { "success" } else { "failure" };
        let fields = json!({
            "schema_version": 1,
            "plan_id": pid.to_string(),
            "stage": "apply.result",
            "decision": decision,
        });
        self.facts.emit("switchyard", "apply.result", decision, fields);

        // Compute total duration
        let duration_ms = t0.elapsed().as_millis() as u64;
        ApplyReport { executed, duration_ms, errors, plan_uuid: Some(pid), rolled_back, rollback_errors }
    }

    pub fn plan_rollback_of(&self, report: &ApplyReport) -> Plan {
        let mut actions: Vec<Action> = Vec::new();
        for act in report.executed.iter().rev() {
            match act {
                Action::EnsureSymlink { target, .. } => {
                    actions.push(Action::RestoreFromBackup { target: target.clone() });
                }
                Action::RestoreFromBackup { .. } => {
                    // Unknown prior state; skip generating an inverse.
                }
            }
        }
        Plan { actions }
    }
}
