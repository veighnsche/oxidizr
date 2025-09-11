//! api/preflight.rs — extracted preflight() implementation

use crate::logging::{FactsEmitter, TS_ZERO};
use crate::types::ids::{action_id, plan_id};
use crate::types::{Action, Plan, PreflightReport};

use super::fs_meta::kind_of;
use super::audit::{emit_preflight_fact, emit_summary, AuditCtx, AuditMode};

pub(crate) fn run<E: FactsEmitter, A: crate::logging::AuditSink>(
    api: &super::Switchyard<E, A>,
    plan: &Plan,
) -> PreflightReport {
    let mut warnings: Vec<String> = Vec::new();
    let mut stops: Vec<String> = Vec::new();

    for act in &plan.actions {
        match act {
            Action::EnsureSymlink { source, target } => {
                if let Err(e) = crate::preflight::ensure_mount_rw_exec(std::path::Path::new("/usr")) {
                    stops.push(format!("/usr not rw+exec: {}", e));
                }
                if let Err(e) = crate::preflight::ensure_mount_rw_exec(&target.as_path()) {
                    stops.push(format!(
                        "target not rw+exec: {} (target={})",
                        e,
                        target.as_path().display()
                    ));
                }
                if let Err(e) = crate::preflight::check_immutable(&target.as_path()) {
                    stops.push(format!(
                        "immutable target: {} (target={})",
                        e,
                        target.as_path().display()
                    ));
                }
                match crate::preflight::check_source_trust(
                    &source.as_path(),
                    api.policy.force_untrusted_source,
                ) {
                    Ok(()) => {}
                    Err(e) => {
                        if api.policy.force_untrusted_source {
                            warnings.push(format!("untrusted source allowed by policy: {}", e));
                        } else {
                            stops.push(format!("untrusted source: {}", e));
                        }
                    }
                }
                if api.policy.strict_ownership {
                    match &api.owner {
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
                if !api.policy.allow_roots.is_empty() {
                    let target_abs = target.as_path();
                    let in_allowed = api
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
                if api
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
                if let Err(e) = crate::preflight::ensure_mount_rw_exec(std::path::Path::new("/usr")) {
                    stops.push(format!("/usr not rw+exec: {}", e));
                }
                if let Err(e) = crate::preflight::ensure_mount_rw_exec(&target.as_path()) {
                    stops.push(format!(
                        "target not rw+exec: {} (target={})",
                        e,
                        target.as_path().display()
                    ));
                }
                if let Err(e) = crate::preflight::check_immutable(&target.as_path()) {
                    stops.push(format!(
                        "immutable target: {} (target={})",
                        e,
                        target.as_path().display()
                    ));
                }
                if !api.policy.allow_roots.is_empty() {
                    let target_abs = target.as_path();
                    let in_allowed = api
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
                if api
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
    let ctx = AuditCtx::new(
        &api.facts as &dyn FactsEmitter,
        pid.to_string(),
        TS_ZERO.to_string(),
        AuditMode { dry_run: true, redact: true },
    );
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
        emit_preflight_fact(
            &ctx,
            &aid.to_string(),
            path.as_deref(),
            &current_kind,
            &planned_kind,
        );
    }
    // Minimal Facts v1: preflight summary
    let decision = if stops.is_empty() { "success" } else { "failure" };
    emit_summary(&ctx, "preflight", decision);

    PreflightReport {
        ok: stops.is_empty(),
        warnings,
        stops,
    }
}
