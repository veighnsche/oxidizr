use serde_json::{json, Value};
use uuid::Uuid;

use crate::logging::{AuditSink, FactsEmitter, StageLogger};
use crate::types::{Action, Plan, PreflightRow};

use crate::logging::audit::AuditCtx;

#[derive(Debug, Default, Clone)]
pub(crate) struct PreflightRowArgs {
    pub path: String,
    pub current_kind: String,
    pub planned_kind: String,
    pub policy_ok: Option<bool>,
    pub provenance: Option<Value>,
    pub notes: Option<Vec<String>>,
    pub preservation: Option<Value>,
    pub preservation_supported: Option<bool>,
    pub restore_ready: Option<bool>,
}

pub(crate) struct RowEmitter<'a, E: FactsEmitter, A: AuditSink> {
    pub api: &'a super::super::Switchyard<E, A>,
    pub plan: &'a Plan,
}

impl<E: FactsEmitter, A: AuditSink> RowEmitter<'_, E, A> {
    pub(super) fn emit_row(
        &self,
        rows: &mut Vec<Value>,
        ctx: &AuditCtx<'_>,
        act: &Action,
        args: PreflightRowArgs,
    ) {
        // Find the stable action_id position
        let idx = self
            .plan
            .actions
            .iter()
            .position(|a| std::ptr::eq(a, act))
            .unwrap_or(0);
        let pid = Uuid::parse_str(&ctx.plan_id).unwrap_or_default();
        let aid = crate::types::ids::action_id(&pid, act, idx);

        // Build typed row and serialize for report rows
        let row = PreflightRow {
            action_id: aid.to_string(),
            path: args.path.clone(),
            current_kind: args.current_kind.clone(),
            planned_kind: args.planned_kind.clone(),
            policy_ok: args.policy_ok,
            provenance: args.provenance.clone(),
            notes: args.notes.clone(),
            preservation: args.preservation.clone(),
            preservation_supported: args.preservation_supported,
            restore_ready: args.restore_ready,
            backup_tag: Some(self.api.policy.backup.tag.clone()),
        };
        if let Ok(value) = serde_json::to_value(row) {
            rows.push(value);
        }

        // Emit fact via facade
        let slog = StageLogger::new(ctx);
        let mut evt = slog
            .preflight()
            .action_id(aid.to_string())
            .path(args.path)
            .field("current_kind", json!(args.current_kind))
            .field("planned_kind", json!(args.planned_kind));
        if let Some(ok) = args.policy_ok {
            evt = evt.field("policy_ok", json!(ok));
        }
        if let Some(p) = args.provenance {
            evt = evt.field("provenance", p);
        }
        if let Some(n) = args.notes {
            evt = evt.field("notes", json!(n));
        }
        if let Some(p) = args.preservation {
            evt = evt.field("preservation", p);
        }
        if let Some(ps) = args.preservation_supported {
            evt = evt.field("preservation_supported", json!(ps));
        }
        // Carry backup tag for traceability per TESTPLAN (long/coreutils/empty tag cases)
        evt = evt.field("backup_tag", json!(self.api.policy.backup.tag.clone()));
        evt.emit_success();
    }
}
