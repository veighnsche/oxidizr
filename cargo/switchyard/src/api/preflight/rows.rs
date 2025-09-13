use serde_json::{json, Value};
use uuid::Uuid;

use crate::logging::{AuditSink, FactsEmitter, StageLogger};
use crate::types::{Action, Plan, PreflightRow};

use crate::logging::audit::AuditCtx;

/// Helper to push a preflight row into the rows vec and emit the corresponding fact.
pub(crate) fn push_row_emit<E: FactsEmitter, A: AuditSink>(
    api: &super::super::Switchyard<E, A>,
    plan: &Plan,
    act: &Action,
    rows: &mut Vec<Value>,
    ctx: &AuditCtx<'_>,
    path: String,
    current_kind: &str,
    planned_kind: &str,
    policy_ok: Option<bool>,
    provenance: Option<Value>,
    notes: Option<Vec<String>>,
    preservation: Option<Value>,
    preservation_supported: Option<bool>,
    restore_ready: Option<bool>,
) {
    // Find the stable action_id position
    let idx = plan
        .actions
        .iter()
        .position(|a| std::ptr::eq(a, act))
        .unwrap_or(0);
    let pid = Uuid::parse_str(&ctx.plan_id).unwrap_or_default();
    let aid = crate::types::ids::action_id(&pid, act, idx);

    // Build typed row and serialize for report rows
    let row = PreflightRow {
        action_id: aid.to_string(),
        path: path.clone(),
        current_kind: current_kind.to_string(),
        planned_kind: planned_kind.to_string(),
        policy_ok,
        provenance: provenance.clone(),
        notes: notes.clone(),
        preservation: preservation.clone(),
        preservation_supported,
        restore_ready,
        backup_tag: Some(api.policy.backup.tag.clone()),
    };
    if let Ok(value) = serde_json::to_value(row) {
        rows.push(value);
    }

    // Emit fact via facade
    let slog = StageLogger::new(ctx);
    let mut evt = slog
        .preflight()
        .action(aid.to_string())
        .path(path)
        .field("current_kind", json!(current_kind))
        .field("planned_kind", json!(planned_kind));
    if let Some(ok) = policy_ok {
        evt = evt.field("policy_ok", json!(ok));
    }
    if let Some(p) = provenance {
        evt = evt.field("provenance", p);
    }
    if let Some(n) = notes {
        evt = evt.field("notes", json!(n));
    }
    if let Some(p) = preservation {
        evt = evt.field("preservation", p);
    }
    if let Some(ps) = preservation_supported {
        evt = evt.field("preservation_supported", json!(ps));
    }
    // Carry backup tag for traceability per TESTPLAN (long/coreutils/empty tag cases)
    evt = evt.field("backup_tag", json!(api.policy.backup.tag.clone()));
    evt.emit_success();
}
