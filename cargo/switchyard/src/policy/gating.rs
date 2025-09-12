use crate::adapters::OwnershipOracle;
use crate::policy::Policy;
use crate::types::{Action, Plan};

/// Centralized evaluation result for a single action under a given Policy.
#[derive(Debug, Default, Clone)]
pub(crate) struct Evaluation {
    pub policy_ok: bool,
    pub stops: Vec<String>,
    pub notes: Vec<String>,
}

/// Evaluate policy gating for a single action.
pub(crate) fn evaluate_action(
    policy: &Policy,
    owner: Option<&dyn OwnershipOracle>,
    act: &Action,
) -> Evaluation {
    let mut stops: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    match act {
        Action::EnsureSymlink { source, target } => {
            // Policy-driven extra mount checks (replaces any hard-coded paths)
            for p in &policy.extra_mount_checks {
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
            if let Ok(hard) = crate::preflight::checks::check_hardlink_hazard(&target.as_path()) {
                if hard {
                    if policy.allow_hardlink_breakage {
                        notes.push("hardlink risk allowed by policy".to_string());
                    } else {
                        stops.push("hardlink risk".to_string());
                        notes.push("hardlink risk".to_string());
                    }
                }
            }
            if let Ok(risk) = crate::preflight::checks::check_suid_sgid_risk(&target.as_path()) {
                if risk {
                    if policy.allow_suid_sgid_mutation {
                        notes.push("suid/sgid risk allowed by policy".to_string());
                    } else {
                        stops.push(format!(
                            "suid/sgid risk: {}",
                            target.as_path().display()
                        ));
                        notes.push("suid/sgid risk".to_string());
                    }
                }
            }
            match crate::preflight::checks::check_source_trust(
                &source.as_path(),
                policy.force_untrusted_source,
            ) {
                Ok(()) => {}
                Err(e) => {
                    if policy.force_untrusted_source {
                        notes.push(format!("untrusted source allowed by policy: {}", e));
                    } else {
                        stops.push(format!("untrusted source: {}", e));
                        notes.push("untrusted source".to_string());
                    }
                }
            }
            if policy.strict_ownership {
                match owner {
                    Some(oracle) => {
                        if let Err(e) = oracle.owner_of(target) {
                            stops.push(format!("strict ownership check failed: {}", e));
                            notes.push("strict ownership check failed".to_string());
                        }
                    }
                    None => {
                        stops.push("strict ownership policy requires OwnershipOracle".to_string());
                        notes.push("missing OwnershipOracle for strict ownership".to_string());
                    }
                }
            }
            if !policy.allow_roots.is_empty() {
                let target_abs = target.as_path();
                let in_allowed = policy.allow_roots.iter().any(|r| target_abs.starts_with(r));
                if !in_allowed {
                    stops.push(format!(
                        "target outside allowed roots: {}",
                        target_abs.display()
                    ));
                    notes.push("target outside allowed roots".to_string());
                }
            }
            if policy
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
            for p in &policy.extra_mount_checks {
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
            if let Ok(risk) = crate::preflight::checks::check_suid_sgid_risk(&target.as_path()) {
                if risk {
                    if policy.allow_suid_sgid_mutation {
                        notes.push("suid/sgid risk allowed by policy".to_string());
                    } else {
                        stops.push("suid/sgid risk".to_string());
                        notes.push("suid/sgid risk".to_string());
                    }
                }
            }
            if !policy.allow_roots.is_empty() {
                let target_abs = target.as_path();
                let in_allowed = policy.allow_roots.iter().any(|r| target_abs.starts_with(r));
                if !in_allowed {
                    stops.push(format!(
                        "target outside allowed roots: {}",
                        target_abs.display()
                    ));
                    notes.push("target outside allowed roots".to_string());
                }
            }
            if policy
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

    Evaluation { policy_ok: stops.is_empty(), stops, notes }
}

/// Compute policy gating errors for a given plan under the current Switchyard policy.
/// This mirrors the gating performed in apply.rs before executing actions.
pub(crate) fn gating_errors(
    policy: &Policy,
    owner: Option<&dyn OwnershipOracle>,
    plan: &Plan,
) -> Vec<String> {
    let mut errs: Vec<String> = Vec::new();

    // Global rescue verification: if required by policy, STOP when unavailable.
    if policy.require_rescue
        && !crate::policy::rescue::verify_rescue_tools_with_exec_min(
            policy.rescue_exec_check,
            policy.rescue_min_count,
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
