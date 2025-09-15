//! Deterministic UUIDv5 identifiers for plans and actions.
//!
//! The UUID namespace is derived from a stable tag (`NS_TAG`) so that
//! `plan_id` and `action_id` are reproducible across runs for the same
//! serialized action sequence.
use std::fmt::Write;
use uuid::Uuid;

use super::{
    plan::{Action, Plan},
    safepath::SafePath,
};
// UUIDv5 namespace tag for deterministic plan/action IDs.
// See SPEC Reproducible v1.1 (Determinism) for guidance.
use crate::constants::NS_TAG;

/// Internal: return the UUID namespace used for deterministic IDs.
fn namespace() -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_URL, NS_TAG.as_bytes())
}

/// Return the relative representation of a `SafePath` to keep IDs independent of roots.
fn sp_rel(p: &SafePath) -> String {
    // Use the relative portion only for determinism across roots
    p.rel().to_string_lossy().to_string()
}

/// Serialize an action into a stable, human-readable string used for UUIDv5 input.
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

/// Compute a deterministic UUIDv5 for a plan by serializing actions in order.
///
/// Two plans with identical action sequences (including ordering) will have the
/// same `plan_id`, independent of the root directories used by `SafePath`.
#[must_use]
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

/// Compute a deterministic UUIDv5 for an action as a function of the plan ID and
/// the action's serialized form, including the stable position index.
#[must_use]
pub fn action_id(plan_id: &Uuid, action: &Action, idx: usize) -> Uuid {
    let mut s = serialize_action(action);
    let _ = write!(s, "#{idx}"); // Ignore formatting errors as they're unlikely in this context
    Uuid::new_v5(plan_id, s.as_bytes())
}
