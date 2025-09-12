use crate::types::report::ApplyReport;
use crate::types::{Action, Plan};

/// Derive an inverse plan from an ApplyReport by reversing executed actions.
///
/// Restore invertibility: for a `RestoreFromBackup` action, we invert to another
/// `RestoreFromBackup` targeting the same path. This relies on the engine having
/// captured a pre-restore snapshot in `handle_restore` when policy enables it.
pub(crate) fn inverse_with_policy(policy: &crate::policy::Policy, report: &ApplyReport) -> Plan {
    let mut actions: Vec<Action> = Vec::new();
    for act in report.executed.iter().rev() {
        match act {
            Action::EnsureSymlink { target, .. } => {
                actions.push(Action::RestoreFromBackup {
                    target: target.clone(),
                });
            }
            Action::RestoreFromBackup { target } => {
                if policy.capture_restore_snapshot {
                    // Invert restore to restore, leveraging the latest pre-restore snapshot
                    actions.push(Action::RestoreFromBackup {
                        target: target.clone(),
                    });
                } else {
                    // Unknown prior state without snapshot; skip.
                }
            }
        }
    }
    Plan { actions }
}
