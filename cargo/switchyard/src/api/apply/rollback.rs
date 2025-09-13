use serde_json::json;

use crate::logging::StageLogger;
use crate::logging::{AuditSink, FactsEmitter};
use crate::types::Action;

pub(crate) fn do_rollback<E: FactsEmitter, A: AuditSink>(
    api: &super::super::Switchyard<E, A>,
    executed: &Vec<Action>,
    dry: bool,
    slog: &StageLogger<'_>,
    rollback_errors: &mut Vec<String>,
) {
    for prev in executed.iter().rev() {
        match prev {
            Action::EnsureSymlink {
                source: _source,
                target,
            } => {
                match crate::fs::restore::restore_file(
                    &target,
                    dry,
                    api.policy.apply.best_effort_restore,
                    &api.policy.backup.tag,
                ) {
                    Ok(()) => {
                        slog.rollback().path(target.as_path().display().to_string()).emit_success();
                    }
                    Err(e) => {
                        rollback_errors.push(format!(
                            "rollback restore {} failed: {}",
                            target.as_path().display(),
                            e
                        ));
                        slog.rollback().path(target.as_path().display().to_string()).emit_failure();
                    }
                }
            }
            Action::RestoreFromBackup { .. } => {
                // No reliable inverse without prior state capture; record informational error.
                rollback_errors.push(
                    "rollback of RestoreFromBackup not supported (no prior state)".to_string(),
                );
                slog.rollback().emit_failure();
            }
        }
    }
}

pub(crate) fn emit_summary(slog: &StageLogger<'_>, rollback_errors: &Vec<String>) {
    let rb_decision = if rollback_errors.is_empty() { "success" } else { "failure" };
    let mut rb_extra = json!({});
    if rb_decision == "failure" {
        if let Some(obj) = rb_extra.as_object_mut() {
            obj.insert(
                "error_id".to_string(),
                json!(crate::api::errors::id_str(crate::api::errors::ErrorId::E_RESTORE_FAILED)),
            );
            obj.insert(
                "exit_code".to_string(),
                json!(crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_RESTORE_FAILED)),
            );
            obj.insert(
                "summary_error_ids".to_string(),
                json!([
                    crate::api::errors::id_str(crate::api::errors::ErrorId::E_RESTORE_FAILED),
                    crate::api::errors::id_str(crate::api::errors::ErrorId::E_POLICY)
                ]),
            );
        }
    }
    if rb_decision == "failure" {
        slog.rollback_summary().merge(rb_extra).emit_failure();
    } else {
        slog.rollback_summary().merge(rb_extra).emit_success();
    }
}
