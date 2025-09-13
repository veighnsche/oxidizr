use crate::logging::{AuditSink, FactsEmitter};

pub(crate) mod ensure_symlink;
pub(crate) mod restore;

/// Small, focused per-action executor.
pub(crate) trait ActionExecutor<E: FactsEmitter, A: AuditSink> {
    fn execute(
        &self,
        api: &super::super::Switchyard<E, A>,
        tctx: &crate::logging::audit::AuditCtx<'_>,
        pid: &uuid::Uuid,
        act: &crate::types::Action,
        idx: usize,
        dry: bool,
    ) -> (
        Option<crate::types::Action>,
        Option<String>,
        super::perf::PerfAgg,
    );
}
