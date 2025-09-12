//! Preflight stage: policy gating, preservation probes, and per-action rows emission.
//!
//! Side-effects:
//! - Emits one preflight fact per action with core fields and optional provenance/notes/preservation.
//! - Emits a preflight summary with a `rescue_profile` status.
//! - Returns a `PreflightReport` with stable row ordering suitable for YAML export via `preflight::to_yaml()`.
//!
//! This module is the stage orchestrator. Low-level helper checks and the YAML
//! exporter live under `crate::preflight::{checks,yaml}`.

use crate::logging::{FactsEmitter, TS_ZERO};
use crate::types::ids::plan_id;
use crate::types::{Action, Plan, PreflightReport};
use serde_json::json;

use crate::fs::meta::{detect_preservation_capabilities, kind_of};
use crate::logging::audit::{AuditCtx, AuditMode};
mod rows;

pub(crate) fn run<E: FactsEmitter, A: crate::logging::AuditSink>(
    api: &super::Switchyard<E, A>,
    plan: &Plan,
) -> PreflightReport {
    let mut warnings: Vec<String> = Vec::new();
    let mut stops: Vec<String> = Vec::new();
    let mut rows: Vec<serde_json::Value> = Vec::new();
    // Shared audit context for preflight stage
    let pid = plan_id(plan);
    let ctx = AuditCtx::new(
        &api.facts as &dyn FactsEmitter,
        pid.to_string(),
        TS_ZERO.to_string(),
        AuditMode {
            dry_run: true,
            redact: true,
        },
    );

    // Global rescue verification: if required by policy, STOP when unavailable.
    let rescue_ok = crate::policy::rescue::verify_rescue_tools_with_exec_min(
        api.policy.rescue_exec_check,
        api.policy.rescue_min_count,
    );
    if api.policy.require_rescue && !rescue_ok {
        stops.push("rescue profile unavailable".to_string());
    }

    for act in &plan.actions {
        match act {
            Action::EnsureSymlink { source, target } => {
                let mut notes: Vec<String> = Vec::new();
                let stops_before = stops.len();
                for p in &api.policy.extra_mount_checks {
                    if let Err(e) = crate::preflight::checks::ensure_mount_rw_exec(p.as_path()) {
                        stops.push(format!("{} not rw+exec: {}", p.display(), e));
                        notes.push(format!("{} not rw+exec", p.display()));
                    }
                }
                if let Err(e) = crate::preflight::checks::ensure_mount_rw_exec(&target.as_path()) {
                    stops.push(format!(
                        "target not rw+exec: {} (target={})",
                        e,
                        target.as_path().display()
                    ));
                    notes.push("target not rw+exec".to_string());
                }
                if let Err(e) = crate::preflight::checks::check_immutable(&target.as_path()) {
                    stops.push(format!(
                        "immutable target: {} (target={})",
                        e,
                        target.as_path().display()
                    ));
                    notes.push("immutable target".to_string());
                }
                if let Ok(hard) = crate::preflight::checks::check_hardlink_hazard(&target.as_path())
                {
                    if hard {
                        if api.policy.allow_hardlink_breakage {
                            warnings.push("hardlink risk allowed by policy".to_string());
                            notes.push("hardlink risk allowed by policy".to_string());
                        } else {
                            stops.push("hardlink risk".to_string());
                            notes.push("hardlink risk".to_string());
                        }
                    }
                }
                if let Ok(risk) = crate::preflight::checks::check_suid_sgid_risk(&target.as_path())
                {
                    if risk {
                        if api.policy.allow_suid_sgid_mutation {
                            warnings.push("suid/sgid risk allowed by policy".to_string());
                            notes.push("suid/sgid risk allowed by policy".to_string());
                        } else {
                            stops.push(format!("suid/sgid risk: {}", target.as_path().display()));
                            notes.push("suid/sgid risk".to_string());
                        }
                    }
                }
                if let Ok(risk) = crate::preflight::checks::check_suid_sgid_risk(&target.as_path())
                {
                    if risk {
                        if api.policy.allow_suid_sgid_mutation {
                            warnings.push("suid/sgid risk allowed by policy".to_string());
                            notes.push("suid/sgid risk allowed by policy".to_string());
                        } else {
                            stops.push(format!("suid/sgid risk: {}", target.as_path().display()));
                            notes.push("suid/sgid risk".to_string());
                        }
                    }
                }
                match crate::preflight::checks::check_source_trust(
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

                let prov = match &api.owner {
                    Some(oracle) => match oracle.owner_of(target) {
                        Ok(info) => {
                            Some(serde_json::json!({"uid":info.uid,"gid":info.gid,"pkg":info.pkg}))
                        }
                        Err(_) => None,
                    },
                    None => None,
                };
                let (preservation, preservation_supported) =
                    detect_preservation_capabilities(&target.as_path());
                if api.policy.require_preservation && !preservation_supported {
                    stops.push("preservation unsupported for target".to_string());
                }
                let policy_ok = stops.len() == stops_before;
                let current_kind = kind_of(&target.as_path());
                rows::push_row_emit(
                    api,
                    plan,
                    act,
                    &mut rows,
                    &ctx,
                    target.as_path().display().to_string(),
                    current_kind,
                    "symlink",
                    Some(policy_ok),
                    prov,
                    if notes.is_empty() { None } else { Some(notes) },
                    Some(preservation),
                    Some(preservation_supported),
                    None,
                );
            }
            Action::RestoreFromBackup { target } => {
                let mut notes: Vec<String> = Vec::new();
                let stops_before = stops.len();
                if let Err(e) =
                    crate::preflight::checks::ensure_mount_rw_exec(std::path::Path::new("/usr"))
                {
                    stops.push(format!("/usr not rw+exec: {}", e));
                    notes.push("/usr not rw+exec".to_string());
                }
                if let Err(e) = crate::preflight::checks::ensure_mount_rw_exec(&target.as_path()) {
                    stops.push(format!(
                        "target not rw+exec: {} (target={})",
                        e,
                        target.as_path().display()
                    ));
                    notes.push("target not rw+exec".to_string());
                }
                if let Err(e) = crate::preflight::checks::check_immutable(&target.as_path()) {
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

                let policy_ok = stops.len() == stops_before;
                let (preservation, preservation_supported) =
                    detect_preservation_capabilities(&target.as_path());
                // Annotate whether backup artifacts are present (payload and/or sidecar)
                let backup_present =
                    crate::fs::has_backup_artifacts(&target.as_path(), &api.policy.backup_tag);
                if api.policy.require_rescue && !backup_present {
                    stops.push("restore requested but no backup artifacts present".to_string());
                    notes.push("no backup artifacts present".to_string());
                }
                rows::push_row_emit(
                    api,
                    plan,
                    act,
                    &mut rows,
                    &ctx,
                    target.as_path().display().to_string(),
                    "unknown".to_string(),
                    "restore_from_backup",
                    Some(policy_ok),
                    None,
                    if notes.is_empty() { None } else { Some(notes) },
                    Some(preservation),
                    Some(preservation_supported),
                    Some(backup_present),
                );
            }
        }
    }

    // Per-action preflight facts are emitted above with extended fields.
    // Minimal Facts v1: preflight summary
    let decision = if stops.is_empty() {
        "success"
    } else {
        "failure"
    };
    // Emit preflight summary with rescue_profile and error mapping when failure
    let prof = if rescue_ok {
        Some("available")
    } else {
        Some("none")
    };
    let mut extra = json!({ "rescue_profile": prof });
    if !stops.is_empty() {
        if let Some(obj) = extra.as_object_mut() {
            obj.insert(
                "error_id".to_string(),
                json!(crate::api::errors::id_str(
                    crate::api::errors::ErrorId::E_POLICY
                )),
            );
            obj.insert(
                "exit_code".to_string(),
                json!(crate::api::errors::exit_code_for(
                    crate::api::errors::ErrorId::E_POLICY
                )),
            );
        }
    }
    crate::logging::audit::emit_summary_extra(&ctx, "preflight", decision, extra);

    // Stable ordering of rows by (path, action_id)
    rows.sort_by(|a, b| {
        let pa = a.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let pb = b.get("path").and_then(|v| v.as_str()).unwrap_or("");
        match pa.cmp(pb) {
            std::cmp::Ordering::Equal => {
                let aa = a.get("action_id").and_then(|v| v.as_str()).unwrap_or("");
                let ab = b.get("action_id").and_then(|v| v.as_str()).unwrap_or("");
                aa.cmp(ab)
            }
            other => other,
        }
    });

    PreflightReport {
        ok: stops.is_empty(),
        warnings,
        stops,
        rows,
    }
}

// YAML exporter intentionally lives in crate::preflight to avoid duplication.
