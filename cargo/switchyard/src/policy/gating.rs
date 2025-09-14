use crate::api::DebugOwnershipOracle;
use crate::policy::types::{RiskLevel, SourceTrustPolicy};
use crate::policy::Policy;
use crate::types::plan::Action;
use crate::types::Plan;

/// Centralized evaluation result for a single action under a given Policy.
#[derive(Debug, Default, Clone)]
pub(crate) struct Evaluation {
    pub policy_ok: bool,
    pub stops: Vec<String>,
    pub notes: Vec<String>,
}

/// Evaluate policy gating for a single action.
#[allow(
    clippy::too_many_lines,
    reason = "Will be decomposed into typed checks in PR8"
)]
pub(crate) fn evaluate_action(
    policy: &Policy,
    owner: Option<&dyn DebugOwnershipOracle>,
    act: &Action,
) -> Evaluation {
    let mut stops: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    match act {
        Action::EnsureSymlink { source, target } => {
            // Policy-driven extra mount checks (replaces any hard-coded paths)
            for p in &policy.apply.extra_mount_checks {
                if let Err(e) = crate::preflight::checks::ensure_mount_rw_exec(p.as_path()) {
                    stops.push(format!("{} not rw+exec: {}", p.display(), e));
                    notes.push(format!("mount: {} not rw+exec", p.display()));
                } else {
                    notes.push(format!("mount ok: {} rw+exec", p.display()));
                }
            }
            if let Err(e) = crate::preflight::checks::ensure_mount_rw_exec(&target.as_path()) {
                stops.push(format!(
                    "target not rw+exec: {} (target={})",
                    e,
                    target.as_path().display()
                ));
                notes.push("mount: target not rw+exec".to_string());
            } else {
                notes.push("mount ok: target rw+exec".to_string());
            }
            if let Err(e) = crate::preflight::checks::check_immutable(&target.as_path()) {
                stops.push(format!(
                    "immutable target: {} (target={})",
                    e,
                    target.as_path().display()
                ));
                notes.push("immutable target".to_string());
            }
            if let Ok(hard) = crate::preflight::checks::check_hardlink_hazard(&target.as_path()) {
                if hard {
                    match policy.risks.hardlinks {
                        RiskLevel::Stop => {
                            stops.push("hardlink risk".to_string());
                            notes.push("hardlink risk".to_string());
                        }
                        RiskLevel::Warn | RiskLevel::Allow => {
                            notes.push("hardlink risk allowed by policy".to_string());
                        }
                    }
                }
            }
            if let Ok(risk) = crate::preflight::checks::check_suid_sgid_risk(&target.as_path()) {
                if risk {
                    match policy.risks.suid_sgid {
                        RiskLevel::Stop => {
                            stops.push(format!("suid/sgid risk: {}", target.as_path().display()));
                            notes.push("suid/sgid risk".to_string());
                        }
                        RiskLevel::Warn | RiskLevel::Allow => {
                            notes.push("suid/sgid risk allowed by policy".to_string());
                        }
                    }
                }
            }
            match crate::preflight::checks::check_source_trust(
                &source.as_path(),
                policy.risks.source_trust != SourceTrustPolicy::RequireTrusted,
            ) {
                Ok(()) => {}
                Err(e) => {
                    if policy.risks.source_trust == SourceTrustPolicy::RequireTrusted {
                        stops.push(format!("untrusted source: {e}"));
                        notes.push("untrusted source".to_string());
                    } else {
                        notes.push(format!("untrusted source allowed by policy: {e}"));
                    }
                }
            }
            if policy.risks.ownership_strict {
                if let Some(oracle) = owner {
                    if let Err(e) = oracle.owner_of(target) {
                        stops.push(format!("strict ownership check failed: {e}"));
                        notes.push("strict ownership check failed".to_string());
                    }
                } else {
                    stops.push("strict ownership policy requires OwnershipOracle".to_string());
                    notes.push("missing OwnershipOracle for strict ownership".to_string());
                }
            }
            if !policy.scope.allow_roots.is_empty() {
                let target_abs = target.as_path();
                let in_allowed = policy
                    .scope
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
            if policy
                .scope
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
        }
        Action::RestoreFromBackup { target } => {
            for p in &policy.apply.extra_mount_checks {
                if let Err(e) = crate::preflight::checks::ensure_mount_rw_exec(p.as_path()) {
                    stops.push(format!("{} not rw+exec: {}", p.display(), e));
                    notes.push(format!("mount: {} not rw+exec", p.display()));
                } else {
                    notes.push(format!("mount ok: {} rw+exec", p.display()));
                }
            }
            if let Err(e) = crate::preflight::checks::ensure_mount_rw_exec(&target.as_path()) {
                stops.push(format!(
                    "target not rw+exec: {} (target={})",
                    e,
                    target.as_path().display()
                ));
                notes.push("mount: target not rw+exec".to_string());
            } else {
                notes.push("mount ok: target rw+exec".to_string());
            }
            if let Err(e) = crate::preflight::checks::check_immutable(&target.as_path()) {
                stops.push(format!(
                    "immutable target: {} (target={})",
                    e,
                    target.as_path().display()
                ));
                notes.push("immutable target".to_string());
            }
            if let Ok(risk) = crate::preflight::checks::check_suid_sgid_risk(&target.as_path()) {
                if risk {
                    match policy.risks.suid_sgid {
                        RiskLevel::Stop => {
                            stops.push("suid/sgid risk".to_string());
                            notes.push("suid/sgid risk".to_string());
                        }
                        RiskLevel::Warn | RiskLevel::Allow => {
                            notes.push("suid/sgid risk allowed by policy".to_string());
                        }
                    }
                }
            }
            if !policy.scope.allow_roots.is_empty() {
                let target_abs = target.as_path();
                let in_allowed = policy
                    .scope
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
            if policy
                .scope
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
        }
    }

    Evaluation {
        policy_ok: stops.is_empty(),
        stops,
        notes,
    }
}

/// Compute policy gating errors for a given plan under the current Switchyard policy.
/// This mirrors the gating performed in apply.rs before executing actions.
pub(crate) fn gating_errors(
    policy: &Policy,
    owner: Option<&dyn DebugOwnershipOracle>,
    plan: &Plan,
) -> Vec<String> {
    let mut errs: Vec<String> = Vec::new();

    // Global rescue verification: if required by policy, STOP when unavailable.
    if policy.rescue.require
        && !crate::policy::rescue::verify_rescue_tools_with_exec_min(
            policy.rescue.exec_check,
            policy.rescue.min_count,
        )
    {
        errs.push("rescue profile unavailable".to_string());
    }

    for act in &plan.actions {
        let eval = evaluate_action(policy, owner, act);
        errs.extend(eval.stops);
    }

    errs
}
