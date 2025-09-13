//! Preflight stage: policy gating, preservation probes, and per-action rows emission.
//!
//! Side-effects:
//! - Emits one preflight fact per action with core fields and optional provenance/notes/preservation.
//! - Emits a preflight summary with a `rescue_profile` status.
//! - Returns a `PreflightReport` with stable row ordering suitable for YAML export via `preflight::to_yaml()`.
//!
//! This module is the stage orchestrator. Low-level helper checks and the YAML
//! exporter live under `crate::preflight::{checks,yaml}`.

use crate::logging::audit::new_run_id;
use crate::logging::{FactsEmitter, TS_ZERO};
use crate::types::ids::plan_id;
use crate::types::{Action, Plan, PreflightReport};
use serde_json::json;

use crate::fs::meta::{detect_preservation_capabilities, kind_of};
use crate::logging::audit::{AuditCtx, AuditMode};
use crate::policy::gating;
mod rows;

#[allow(clippy::too_many_lines, reason = "deferred refactoring")]
pub(crate) fn run<E: FactsEmitter, A: crate::logging::AuditSink>(
    api: &super::Switchyard<E, A>,
    plan: &Plan,
) -> PreflightReport {
    let mut warnings: Vec<String> = Vec::new();
    let mut stops: Vec<String> = Vec::new();
    let mut rows: Vec<serde_json::Value> = Vec::new();
    // Shared audit context for preflight stage
    let pid = plan_id(plan);
    let run_id = new_run_id();
    let ctx = AuditCtx::new(
        &api.facts,
        pid.to_string(),
        run_id,
        TS_ZERO.to_string(),
        AuditMode {
            dry_run: true,
            redact: true,
        },
    );

    // Global rescue verification: if required by policy, STOP when unavailable.
    let rescue_ok = crate::policy::rescue::verify_rescue_tools_with_exec_min(
        api.policy.rescue.exec_check,
        api.policy.rescue.min_count,
    );
    if api.policy.rescue.require && !rescue_ok {
        stops.push("rescue profile unavailable".to_string());
    }

    for act in &plan.actions {
        match act {
            Action::EnsureSymlink { target, .. } => {
                let eval = gating::evaluate_action(&api.policy, api.owner.as_deref(), act);
                // Aggregate stops for summary
                if !eval.stops.is_empty() {
                    stops.extend(eval.stops.clone());
                }
                // Warnings: promote policy-allowed notes as warnings
                warnings.extend(
                    eval.notes
                        .iter()
                        .filter(|n| n.contains("allowed by policy"))
                        .cloned(),
                );
                // Provenance best-effort
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
                if matches!(
                    api.policy.durability.preservation,
                    crate::policy::types::PreservationPolicy::RequireBasic
                ) && !preservation_supported
                {
                    stops.push("preservation unsupported for target".to_string());
                }
                let current_kind = kind_of(&target.as_path());
                rows::push_row_emit(
                    api,
                    plan,
                    act,
                    &mut rows,
                    &ctx,
                    target.as_path().display().to_string(),
                    &current_kind,
                    "symlink",
                    Some(eval.policy_ok),
                    prov,
                    if eval.notes.is_empty() {
                        None
                    } else {
                        Some(eval.notes)
                    },
                    Some(preservation),
                    Some(preservation_supported),
                    None,
                );
            }
            Action::RestoreFromBackup { target } => {
                let eval = gating::evaluate_action(&api.policy, api.owner.as_deref(), act);
                if !eval.stops.is_empty() {
                    stops.extend(eval.stops.clone());
                }
                warnings.extend(
                    eval.notes
                        .iter()
                        .filter(|n| n.contains("allowed by policy"))
                        .cloned(),
                );
                let (preservation, preservation_supported) =
                    detect_preservation_capabilities(&target.as_path());
                // Annotate whether backup artifacts are present (payload and/or sidecar)
                let backup_present = crate::fs::backup::has_backup_artifacts(
                    &target.as_path(),
                    &api.policy.backup.tag,
                );
                if api.policy.rescue.require && !backup_present {
                    stops.push("restore requested but no backup artifacts present".to_string());
                }
                rows::push_row_emit(
                    api,
                    plan,
                    act,
                    &mut rows,
                    &ctx,
                    target.as_path().display().to_string(),
                    "unknown",
                    "restore_from_backup",
                    Some(eval.policy_ok),
                    None,
                    if eval.notes.is_empty() {
                        None
                    } else {
                        Some(eval.notes)
                    },
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
            let mut chain = vec![crate::api::errors::id_str(
                crate::api::errors::ErrorId::E_POLICY,
            )];
            // Best-effort: co-emit E_OWNERSHIP if any stop references ownership
            if stops.iter().any(|s| s.to_lowercase().contains("ownership")) {
                chain.push(crate::api::errors::id_str(
                    crate::api::errors::ErrorId::E_OWNERSHIP,
                ));
            }
            obj.insert("summary_error_ids".to_string(), json!(chain));
        }
    }
    let slog = crate::logging::StageLogger::new(&ctx);
    match decision {
        "failure" => slog.preflight_summary().merge(&extra).emit_failure(),
        _ => slog.preflight_summary().merge(&extra).emit_success(),
    }

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
            other @ (std::cmp::Ordering::Less | std::cmp::Ordering::Greater) => other,
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
