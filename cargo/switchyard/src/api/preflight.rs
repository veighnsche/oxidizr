//! Preflight stage: policy gating, preservation probes, and per-action rows emission.
//!
//! Side-effects:
//! - Emits one preflight fact per action with core fields and optional provenance/notes/preservation.
//! - Emits a preflight summary with a `rescue_profile` status.
//! - Returns a `PreflightReport` with stable row ordering suitable for YAML export via `preflight::to_yaml()`.

use crate::logging::{FactsEmitter, TS_ZERO};
use crate::types::ids::{action_id, plan_id};
use crate::types::{Action, Plan, PreflightReport};
use serde_json::json;

use super::fs_meta::{kind_of, detect_preservation_capabilities};
use super::audit::{emit_preflight_fact_ext, AuditCtx, AuditMode};

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
        AuditMode { dry_run: true, redact: true },
    );

    // Global rescue verification: if required by policy, STOP when unavailable.
    let rescue_ok = crate::rescue::verify_rescue_tools_with_exec_min(
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
                let (preservation, preservation_supported) = detect_preservation_capabilities(&target.as_path());
                if api.policy.require_preservation && !preservation_supported {
                    stops.push("preservation unsupported for target".to_string());
                }
                let policy_ok = stops.len() == stops_before;
                // Build preflight row for report
                let current_kind = kind_of(&target.as_path());
                let mut row = json!({
                    "action_id": aid.to_string(),
                    "path": target.as_path().display().to_string(),
                    "current_kind": current_kind,
                    "planned_kind": "symlink",
                    "policy_ok": policy_ok,
                });
                if let Some(p) = prov.as_ref() {
                    if let Some(o) = row.as_object_mut() { o.insert("provenance".into(), p.clone()); }
                }
                if !notes.is_empty() {
                    if let Some(o) = row.as_object_mut() { o.insert("notes".into(), json!(notes)); }
                }
                if let Some(o) = row.as_object_mut() { o.insert("preservation".into(), preservation.clone()); }
                if let Some(o) = row.as_object_mut() { o.insert("preservation_supported".into(), json!(preservation_supported)); }
                rows.push(row);
                emit_preflight_fact_ext(
                    &ctx,
                    &aid.to_string(),
                    Some(target.as_path().display().to_string()),
                    &current_kind,
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
                // Annotate whether backup artifacts are present (payload and/or sidecar)
                let backup_present = crate::fs::backup::has_backup_artifacts(&target.as_path(), &api.policy.backup_tag);
                if api.policy.require_rescue && !backup_present {
                    stops.push("restore requested but no backup artifacts present".to_string());
                    notes.push("no backup artifacts present".to_string());
                }
                // Build preflight row for report
                let mut row = json!({
                    "action_id": aid.to_string(),
                    "path": target.as_path().display().to_string(),
                    "current_kind": "unknown",
                    "planned_kind": "restore_from_backup",
                    "policy_ok": policy_ok,
                    "backup_present": backup_present,
                });
                if !notes.is_empty() { if let Some(o) = row.as_object_mut() { o.insert("notes".into(), json!(notes)); } }
                if let Some(o) = row.as_object_mut() { o.insert("preservation".into(), preservation.clone()); }
                if let Some(o) = row.as_object_mut() { o.insert("preservation_supported".into(), json!(preservation_supported)); }
                rows.push(row);
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
    // Emit preflight summary with rescue_profile for visibility
    let prof = if rescue_ok { Some("available") } else { Some("none") };
    let extra = json!({ "rescue_profile": prof });
    super::audit::emit_summary_extra(&ctx, "preflight", decision, extra);

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

    PreflightReport { ok: stops.is_empty(), warnings, stops, rows }
}

// YAML exporter intentionally lives in crate::preflight to avoid duplication.
