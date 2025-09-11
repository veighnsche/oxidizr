/// Placeholder

use std::time::Instant;

use crate::{preflight, fs};
use crate::logging::{FactsEmitter, AuditSink};
use crate::policy::Policy;
use crate::types::{PlanInput, Plan, Action, PreflightReport, ApplyMode, ApplyReport};

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
        Plan { actions }
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
                    if let Err(e) = preflight::ensure_mount_rw_exec(target.as_path()) {
                        stops.push(format!("target not rw+exec: {} (target={})", e, target.as_path().display()));
                    }
                    if let Err(e) = preflight::check_immutable(target.as_path()) {
                        stops.push(format!("immutable target: {} (target={})", e, target.as_path().display()));
                    }
                    match preflight::check_source_trust(source.as_path(), self.policy.force_untrusted_source) {
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
                    if let Err(e) = preflight::ensure_mount_rw_exec(target.as_path()) {
                        stops.push(format!("target not rw+exec: {} (target={})", e, target.as_path().display()));
                    }
                    if let Err(e) = preflight::check_immutable(target.as_path()) {
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

        PreflightReport { ok: stops.is_empty(), warnings, stops }
    }

    pub fn apply(&self, plan: &Plan, mode: ApplyMode) -> ApplyReport {
        let t0 = Instant::now();
        let mut executed: Vec<Action> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        let dry = matches!(mode, ApplyMode::DryRun);

        for act in &plan.actions {
            match act {
                Action::EnsureSymlink { source, target } => {
                    match fs::replace_file_with_symlink(source.as_path(), target.as_path(), dry) {
                        Ok(()) => executed.push(act.clone()),
                        Err(e) => errors.push(format!(
                            "symlink {} -> {} failed: {}",
                            source.as_path().display(),
                            target.as_path().display(),
                            e
                        )),
                    }
                }
                Action::RestoreFromBackup { target } => {
                    match fs::restore_file(target.as_path(), dry, self.policy.force_restore_best_effort) {
                        Ok(()) => executed.push(act.clone()),
                        Err(e) => errors.push(format!(
                            "restore {} failed: {}",
                            target.as_path().display(),
                            e
                        )),
                    }
                }
            }
        }

        let duration_ms = t0.elapsed().as_millis() as u64;
        ApplyReport { executed, duration_ms, errors }
    }
}
