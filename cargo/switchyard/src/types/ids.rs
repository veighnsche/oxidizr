use uuid::Uuid;

use super::{
    plan::{Action, Plan},
    safepath::SafePath,
};
// UUIDv5 namespace tag for deterministic plan/action IDs.
// See SPEC Reproducible v1.1 (Determinism) for guidance.
use crate::constants::NS_TAG;

fn namespace() -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_URL, NS_TAG.as_bytes())
}

fn sp_rel(p: &SafePath) -> String {
    // Use the relative portion only for determinism across roots
    p.rel().to_string_lossy().to_string()
}

fn serialize_action(a: &Action) -> String {
    match a {
        Action::EnsureSymlink { source, target } => {
            format!("E:{}->{}", sp_rel(source), sp_rel(target))
        }
        Action::RestoreFromBackup { target } => {
            format!("R:{}", sp_rel(target))
        }
    }
}

pub fn plan_id(plan: &Plan) -> Uuid {
    let ns = namespace();
    // Deterministic serialization in action order
    let mut s = String::new();
    for a in &plan.actions {
        s.push_str(&serialize_action(a));
        s.push('\n');
    }
    Uuid::new_v5(&ns, s.as_bytes())
}

pub fn action_id(plan_id: &Uuid, action: &Action, idx: usize) -> Uuid {
    let mut s = serialize_action(action);
    s.push_str(&format!("#{}", idx));
    Uuid::new_v5(plan_id, s.as_bytes())
}
