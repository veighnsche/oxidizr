use crate::logging::{AuditSink, FactsEmitter};
use crate::types::{Action, Plan};

/// Compute policy gating errors for a given plan under the current Switchyard policy.
/// This mirrors the gating performed in apply.rs before executing actions.
pub(crate) fn gating_errors<E: FactsEmitter, A: AuditSink>(
    api: &super::super::Switchyard<E, A>,
    plan: &Plan,
) -> Vec<String> {
    let mut gating_errors: Vec<String> = Vec::new();

    // Global rescue verification: if required by policy, STOP when unavailable.
    if api.policy.require_rescue
        && !crate::rescue::verify_rescue_tools_with_exec_min(
            api.policy.rescue_exec_check,
            api.policy.rescue_min_count,
        )
    {
        gating_errors.push("rescue profile unavailable".to_string());
    }

    for act in &plan.actions {
        match act {
            Action::EnsureSymlink { source, target } => {
                if let Err(e) = crate::preflight::ensure_mount_rw_exec(std::path::Path::new("/usr")) {
                    gating_errors.push(format!("/usr not rw+exec: {}", e));
                }
                if let Err(e) = crate::preflight::ensure_mount_rw_exec(&target.as_path()) {
                    gating_errors.push(format!(
                        "target not rw+exec: {} (target={})",
                        e,
                        target.as_path().display()
                    ));
                }
                if let Err(e) = crate::preflight::check_immutable(&target.as_path()) {
                    gating_errors.push(format!(
                        "immutable target: {} (target={})",
                        e,
                        target.as_path().display()
                    ));
                }
                if let Err(e) = crate::preflight::check_source_trust(
                    &source.as_path(),
                    api.policy.force_untrusted_source,
                ) {
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
                            gating_errors.push(
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
                        gating_errors.push(format!(
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
                    gating_errors.push(format!(
                        "target in forbidden path: {}",
                        target.as_path().display()
                    ));
                }
            }
            Action::RestoreFromBackup { target } => {
                if let Err(e) = crate::preflight::ensure_mount_rw_exec(std::path::Path::new("/usr")) {
                    gating_errors.push(format!("/usr not rw+exec: {}", e));
                }
                if let Err(e) = crate::preflight::ensure_mount_rw_exec(&target.as_path()) {
                    gating_errors.push(format!(
                        "target not rw+exec: {} (target={})",
                        e,
                        target.as_path().display()
                    ));
                }
                if let Err(e) = crate::preflight::check_immutable(&target.as_path()) {
                    gating_errors.push(format!(
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
                        gating_errors.push(format!(
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
                    gating_errors.push(format!(
                        "target in forbidden path: {}",
                        target.as_path().display()
                    ));
                }
            }
        }
    }

    gating_errors
}
