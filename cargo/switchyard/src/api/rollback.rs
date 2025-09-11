use crate::types::{Action, Plan};
use crate::types::report::ApplyReport;

/// Derive an inverse plan from an ApplyReport by reversing executed actions.
pub(super) fn inverse(report: &ApplyReport) -> Plan {
    let mut actions: Vec<Action> = Vec::new();
    for act in report.executed.iter().rev() {
        match act {
            Action::EnsureSymlink { target, .. } => {
                actions.push(Action::RestoreFromBackup { target: target.clone() });
            }
            Action::RestoreFromBackup { .. } => {
                // Unknown prior state; skip generating an inverse.
            }
        }
    }
    Plan { actions }
}
