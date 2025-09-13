use uuid::Uuid;

use crate::logging::{AuditSink, FactsEmitter};
use crate::types::Action;

use super::perf::PerfAgg;
use crate::api::apply::executors::ActionExecutor;
use crate::logging::audit::AuditCtx;
use crate::logging::StageLogger;

/// Handle an `EnsureSymlink` action: perform the operation and emit per-action facts.
/// Returns (`executed_action_if_success`, `error_message_if_failure`).
pub(crate) fn handle_ensure_symlink<E: FactsEmitter, A: AuditSink>(
    api: &super::super::Switchyard<E, A>,
    tctx: &AuditCtx<'_>,
    pid: &Uuid,
    act: &Action,
    idx: usize,
    dry: bool,
    _slog: &StageLogger<'_>,
) -> (Option<Action>, Option<String>, PerfAgg) {
    let exec = super::executors::ensure_symlink::EnsureSymlinkExec;
    exec.execute(api, tctx, pid, act, idx, dry)
}

/// Handle a `RestoreFromBackup` action: perform restore and emit per-action facts.
/// Returns (`executed_action_if_success`, `error_message_if_failure`).
pub(crate) fn handle_restore<E: FactsEmitter, A: AuditSink>(
    api: &super::super::Switchyard<E, A>,
    tctx: &AuditCtx<'_>,
    pid: &Uuid,
    act: &Action,
    idx: usize,
    dry: bool,
    _slog: &StageLogger<'_>,
) -> (Option<Action>, Option<String>, PerfAgg) {
    let exec = super::executors::restore::RestoreFromBackupExec;
    exec.execute(api, tctx, pid, act, idx, dry)
}
