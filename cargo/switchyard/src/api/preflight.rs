//! api/preflight.rs — extracted preflight() implementation

use crate::logging::{FactsEmitter, TS_ZERO};
use crate::types::ids::{action_id, plan_id};
use crate::types::{Action, Plan, PreflightReport};

use super::fs_meta::{kind_of, detect_preservation_capabilities};
use super::audit::{emit_preflight_fact_ext, emit_summary, AuditCtx, AuditMode};

pub(crate) fn run<E: FactsEmitter, A: crate::logging::AuditSink>(
    api: &super::Switchyard<E, A>,
    plan: &Plan,
) -> PreflightReport {
    let mut warnings: Vec<String> = Vec::new();
    let mut stops: Vec<String> = Vec::new();
    // Shared audit context for preflight stage
    let pid = plan_id(plan);
    let ctx = AuditCtx::new(
        &api.facts as &dyn FactsEmitter,
        pid.to_string(),
        TS_ZERO.to_string(),
        AuditMode { dry_run: true, redact: true },
    );

    for act in &plan.actions {
        match act {
            Action::EnsureSymlink { source, target } => {
                let mut notes: Vec<String> = Vec::new();
                let stops_before = stops.len();
                if let Err(e) = crate::preflight::ensure_mount_rw_exec(std::path::Path::new("/usr")) {
                    stops.push(format!("/usr not rw+exec: {}", e));
                    notes.push("/usr not rw+exec".to_string());
                }
                if let Err(e) = crate::preflight::ensure_mount_rw_exec(&target.as_path()) {
                    stops.push(format!(
                        "target not rw+exec: {} (target={})",
                        e,
                        target.as_path().display()
                    ));
                    notes.push("target not rw+exec".to_string());
                }
                if let Err(e) = crate::preflight::check_immutable(&target.as_path()) {
                    stops.push(format!(
                        "immutable target: {} (target={})",
                        e,
                        target.as_path().display()
                    ));
                    notes.push("immutable target".to_string());
                }
                match crate::preflight::check_source_trust(
                    &source.as_path(),
                    api.policy.force_untrusted_source,
                ) {
                    Ok(()) => {}
                    Err(e) => {
                        if api.policy.force_untrusted_source {
                            warnings.push(format!("untrusted source allowed by policy: {}", e));
                            notes.push("untrusted source allowed by policy".to_string());
                        } else {
                            stops.push(format!("untrusted source: {}", e));
                            notes.push("untrusted source".to_string());
                        }
                    }
                }
                if api.policy.strict_ownership {
                    match &api.owner {
                        Some(oracle) => {
                            if let Err(e) = oracle.owner_of(target) {
                                stops.push(format!("strict ownership check failed: {}", e));
                                notes.push("strict ownership check failed".to_string());
                            }
                        }
                        None => {
                            stops.push(
                                "strict ownership policy requires OwnershipOracle".to_string(),
                            );
                            notes.push("missing OwnershipOracle for strict ownership".to_string());
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
                        notes.push("target outside allowed roots".to_string());
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
                    notes.push("target in forbidden path".to_string());
                }

                let idx = plan.actions.iter().position(|a| std::ptr::eq(a, act)).unwrap_or(0);
                let aid = action_id(&pid, act, idx);
                let prov = match &api.owner {
                    Some(oracle) => match oracle.owner_of(target) {
                        Ok(info) => Some(serde_json::json!({"uid":info.uid,"gid":info.gid,"pkg":info.pkg})),
                        Err(_) => None,
                    },
                    None => None,
                };
                let policy_ok = stops.len() == stops_before;
                let (preservation, preservation_supported) = detect_preservation_capabilities(&target.as_path());
                emit_preflight_fact_ext(
                    &ctx,
                    &aid.to_string(),
                    Some(target.as_path().display().to_string()),
                    &kind_of(&target.as_path()),
                    "symlink",
                    Some(policy_ok),
                    prov,
                    if notes.is_empty() { None } else { Some(notes) },
                    Some(preservation),
                    Some(preservation_supported),
                );
            }
            Action::RestoreFromBackup { target } => {
                let mut notes: Vec<String> = Vec::new();
                let stops_before = stops.len();
                if let Err(e) = crate::preflight::ensure_mount_rw_exec(std::path::Path::new("/usr")) {
                    stops.push(format!("/usr not rw+exec: {}", e));
                    notes.push("/usr not rw+exec".to_string());
                }
                if let Err(e) = crate::preflight::ensure_mount_rw_exec(&target.as_path()) {
                    stops.push(format!(
                        "target not rw+exec: {} (target={})",
                        e,
                        target.as_path().display()
                    ));
                    notes.push("target not rw+exec".to_string());
                }
                if let Err(e) = crate::preflight::check_immutable(&target.as_path()) {
                    stops.push(format!(
                        "immutable target: {} (target={})",
                        e,
                        target.as_path().display()
                    ));
                    notes.push("immutable target".to_string());
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
                        notes.push("target outside allowed roots".to_string());
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
                    notes.push("target in forbidden path".to_string());
                }

                let idx = plan.actions.iter().position(|a| std::ptr::eq(a, act)).unwrap_or(0);
                let aid = action_id(&pid, act, idx);
                let policy_ok = stops.len() == stops_before;
                let (preservation, preservation_supported) = detect_preservation_capabilities(&target.as_path());
                emit_preflight_fact_ext(
                    &ctx,
                    &aid.to_string(),
                    Some(target.as_path().display().to_string()),
                    &"unknown".to_string(),
                    "restore_from_backup",
                    Some(policy_ok),
                    None,
                    if notes.is_empty() { None } else { Some(notes) },
                    Some(preservation),
                    Some(preservation_supported),
                );
            }
        }
    }

    // Per-action preflight facts are emitted above with extended fields.
    // Minimal Facts v1: preflight summary
    let decision = if stops.is_empty() { "success" } else { "failure" };
    emit_summary(&ctx, "preflight", decision);

    PreflightReport {
        ok: stops.is_empty(),
        warnings,
        stops,
    }
}
