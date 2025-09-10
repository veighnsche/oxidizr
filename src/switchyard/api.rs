use std::path::PathBuf;
use std::time::Instant;

/// Structured facts emitter (machine-readable events). Kept minimal for phase 1.
pub trait FactsEmitter {
    fn emit(&self, _subsystem: &str, _event: &str, _decision: &str, _fields: serde_json::Value) {}
}

/// Human-readable audit sink (lines/messages). Kept minimal for phase 1.
pub trait AuditSink {
    fn log(&self, _level: log::Level, _msg: &str) {}
}

#[derive(Clone, Debug, Default)]
pub struct Policy {
    pub allow_roots: Vec<PathBuf>,
    pub forbid_paths: Vec<PathBuf>,
    pub strict_ownership: bool,
    pub force_untrusted_source: bool,
    pub force_restore_best_effort: bool,
}

#[derive(Clone, Debug)]
pub enum ApplyMode {
    DryRun,
    Commit,
}

impl Default for ApplyMode {
    fn default() -> Self { ApplyMode::DryRun }
}

#[derive(Clone, Debug, Default)]
pub struct LinkRequest { pub source: PathBuf, pub target: PathBuf }

#[derive(Clone, Debug, Default)]
pub struct RestoreRequest { pub target: PathBuf }

#[derive(Clone, Debug, Default)]
pub struct PlanInput {
    pub link: Vec<LinkRequest>,
    pub restore: Vec<RestoreRequest>,
}

#[derive(Clone, Debug)]
pub enum Action {
    EnsureSymlink { source: PathBuf, target: PathBuf },
    RestoreFromBackup { target: PathBuf },
}

#[derive(Clone, Debug, Default)]
pub struct Plan { pub actions: Vec<Action> }

#[derive(Clone, Debug, Default)]
pub struct PreflightReport {
    pub ok: bool,
    pub warnings: Vec<String>,
    pub stops: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ApplyReport {
    pub executed: Vec<Action>,
    pub skipped: Vec<Action>,
    pub duration_ms: u64,
    pub errors: Vec<String>,
}

pub struct Switchyard<E: FactsEmitter, A: AuditSink> {
    facts: E,
    audit: A,
    policy: Policy,
}

impl<E: FactsEmitter, A: AuditSink> Switchyard<E, A> {
    pub fn new(facts: E, audit: A, policy: Policy) -> Self { Self { facts, audit, policy } }

    /// Build a deterministic plan from inputs and policy. For phase 1, order is stable insertion order.
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

    /// Read-only validations mirroring current fs checks and path safety. Aggregates warnings/stops.
    pub fn preflight(&self, plan: &Plan) -> PreflightReport {
        let mut warnings: Vec<String> = Vec::new();
        let mut stops: Vec<String> = Vec::new();

        for act in &plan.actions {
            match act {
                Action::EnsureSymlink { source, target } => {
                    // Path safety (no traversal)
                    if !crate::symlink::ops::is_safe_path(source) || !crate::symlink::ops::is_safe_path(target) {
                        stops.push(format!("unsafe path (traversal): source={} target={}", source.display(), target.display()));
                        continue;
                    }
                    // Mount & immutability
                    if let Err(e) = crate::system::fs_checks::ensure_mount_rw_exec(std::path::Path::new("/usr")) {
                        stops.push(format!("/usr not rw+exec: {}", e));
                    }
                    if let Err(e) = crate::system::fs_checks::ensure_mount_rw_exec(target.as_path()) {
                        stops.push(format!("target not rw+exec: {} (target={})", e, target.display()));
                    }
                    if let Err(e) = crate::system::fs_checks::check_immutable(target.as_path()) {
                        stops.push(format!("immutable target: {} (target={})", e, target.display()));
                    }
                    // Trust checks (warn/stop per policy)
                    match crate::system::fs_checks::check_source_trust(source.as_path(), self.policy.force_untrusted_source) {
                        Ok(()) => {},
                        Err(e) => {
                            if self.policy.force_untrusted_source {
                                warnings.push(format!("untrusted source allowed by policy: {}", e));
                            } else {
                                stops.push(format!("untrusted source: {}", e));
                            }
                        }
                    }
                    // Allowed roots and forbidden paths (policy)
                    if !self.policy.allow_roots.is_empty() {
                        let in_allowed = self.policy.allow_roots.iter().any(|r| target.starts_with(r));
                        if !in_allowed { stops.push(format!("target outside allowed roots: {}", target.display())); }
                    }
                    if self.policy.forbid_paths.iter().any(|f| target.starts_with(f)) {
                        stops.push(format!("target in forbidden path: {}", target.display()));
                    }
                }
                Action::RestoreFromBackup { target } => {
                    if !crate::symlink::ops::is_safe_path(target) {
                        stops.push(format!("unsafe target path (traversal): {}", target.display()));
                        continue;
                    }
                    if let Err(e) = crate::system::fs_checks::ensure_mount_rw_exec(std::path::Path::new("/usr")) {
                        stops.push(format!("/usr not rw+exec: {}", e));
                    }
                    if let Err(e) = crate::system::fs_checks::ensure_mount_rw_exec(target.as_path()) {
                        stops.push(format!("target not rw+exec: {} (target={})", e, target.display()));
                    }
                    if let Err(e) = crate::system::fs_checks::check_immutable(target.as_path()) {
                        stops.push(format!("immutable target: {} (target={})", e, target.display()));
                    }
                    if !self.policy.allow_roots.is_empty() {
                        let in_allowed = self.policy.allow_roots.iter().any(|r| target.starts_with(r));
                        if !in_allowed { stops.push(format!("target outside allowed roots: {}", target.display())); }
                    }
                    if self.policy.forbid_paths.iter().any(|f| target.starts_with(f)) {
                        stops.push(format!("target in forbidden path: {}", target.display()));
                    }
                }
            }
        }

        PreflightReport { ok: stops.is_empty(), warnings, stops }
    }

    /// Apply actions. For phase 1, delegate to existing symlink ops with dry-run mapping.
    pub fn apply(&self, plan: &Plan, mode: ApplyMode) -> ApplyReport {
        let t0 = Instant::now();
        let mut executed: Vec<Action> = Vec::new();
        let mut skipped: Vec<Action> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        let dry = matches!(mode, ApplyMode::DryRun);

        for act in &plan.actions {
            match act {
                Action::EnsureSymlink { source, target } => {
                    match crate::symlink::replace_file_with_symlink(source.as_path(), target.as_path(), dry) {
                        Ok(()) => executed.push(act.clone()),
                        Err(e) => errors.push(format!("symlink {} -> {} failed: {}", source.display(), target.display(), e)),
                    }
                }
                Action::RestoreFromBackup { target } => {
                    match crate::symlink::restore_file(target.as_path(), dry, self.policy.force_restore_best_effort) {
                        Ok(()) => executed.push(act.clone()),
                        Err(e) => errors.push(format!("restore {} failed: {}", target.display(), e)),
                    }
                }
            }
        }

        let duration_ms = t0.elapsed().as_millis() as u64;
        ApplyReport { executed, skipped, duration_ms, errors }
    }
}
