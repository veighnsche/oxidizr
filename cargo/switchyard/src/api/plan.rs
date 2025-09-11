//! api/plan.rs — extracted plan() implementation

use crate::logging::FactsEmitter;
use crate::types::ids::{action_id, plan_id};
use crate::types::{Action, Plan, PlanInput};

use super::audit::{emit_plan_fact, AuditCtx, AuditMode};

/// Build a deterministic plan from input and emit per-action plan facts.
pub(super) fn build<E: FactsEmitter, A: crate::logging::AuditSink>(
    api: &super::Switchyard<E, A>,
    input: PlanInput,
) -> Plan {
    let mut actions: Vec<Action> = Vec::new();
    for l in input.link {
        actions.push(Action::EnsureSymlink {
            source: l.source,
            target: l.target,
        });
    }
    for r in input.restore {
        actions.push(Action::RestoreFromBackup { target: r.target });
    }
    // Stable ordering: sort actions by deterministic key (target rel path), then by kind
    actions.sort_by(|a, b| {
        let ka = match a {
            Action::EnsureSymlink { target, .. } => (0u8, target.rel().to_string_lossy().to_string()),
            Action::RestoreFromBackup { target } => {
                (1u8, target.rel().to_string_lossy().to_string())
            }
        };
        let kb = match b {
            Action::EnsureSymlink { target, .. } => (0u8, target.rel().to_string_lossy().to_string()),
            Action::RestoreFromBackup { target } => {
                (1u8, target.rel().to_string_lossy().to_string())
            }
        };
        ka.cmp(&kb)
    });
    let plan = Plan { actions };

    // Emit per-action plan facts using telemetry helper
    let pid_uuid = plan_id(&plan);
    let pid = pid_uuid.to_string();
    let tctx = AuditCtx::new(
        &api.facts as &dyn FactsEmitter,
        pid.clone(),
        crate::logging::TS_ZERO.to_string(),
        AuditMode {
            dry_run: true,
            redact: true,
        },
    );
    for (idx, act) in plan.actions.iter().enumerate() {
        let aid = action_id(&pid_uuid, act, idx).to_string();
        let path = match act {
            Action::EnsureSymlink { target, .. } => Some(target.as_path().display().to_string()),
            Action::RestoreFromBackup { target } => Some(target.as_path().display().to_string()),
        };
        emit_plan_fact(&tctx, &aid, path.as_deref());
    }

    plan
}
