use serde_json::{json, Value};
use uuid::Uuid;

use crate::logging::{AuditSink, FactsEmitter, StageLogger};
use crate::types::{Action, Plan};

use crate::logging::audit::AuditCtx;

/// Helper to push a preflight row into the rows vec and emit the corresponding fact.
pub(crate) fn push_row_emit<E: FactsEmitter, A: AuditSink>(
    _api: &super::super::Switchyard<E, A>,
    plan: &Plan,
    act: &Action,
    rows: &mut Vec<Value>,
    ctx: &AuditCtx,
    path: String,
    current_kind: String,
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

    // Build the row and push
    let mut row = json!({
        "action_id": aid.to_string(),
        "path": path,
        "current_kind": current_kind,
        "planned_kind": planned_kind,
    });
    if let Some(ok) = policy_ok {
        if let Some(o) = row.as_object_mut() {
            o.insert("policy_ok".into(), json!(ok));
        }
    }
    if let Some(p) = provenance.as_ref() {
        if let Some(o) = row.as_object_mut() {
            o.insert("provenance".into(), p.clone());
        }
    }
    if let Some(ns) = notes.as_ref() {
        if let Some(o) = row.as_object_mut() {
            o.insert("notes".into(), json!(ns));
        }
    }
    if let Some(p) = preservation.as_ref() {
        if let Some(o) = row.as_object_mut() {
            o.insert("preservation".into(), p.clone());
        }
    }
    if let Some(ps) = preservation_supported {
        if let Some(o) = row.as_object_mut() {
            o.insert("preservation_supported".into(), json!(ps));
        }
    }
    if let Some(rr) = restore_ready {
        if let Some(o) = row.as_object_mut() {
            o.insert("restore_ready".into(), json!(rr));
        }
    }
    rows.push(row);

    // Emit fact via facade
    let slog = StageLogger::new(ctx);
    let mut evt = slog
        .preflight()
        .action(aid.to_string())
        .path(path)
        .field("current_kind", json!(current_kind))
        .field("planned_kind", json!(planned_kind));
    if let Some(ok) = policy_ok { evt = evt.field("policy_ok", json!(ok)); }
    if let Some(p) = provenance { evt = evt.field("provenance", p); }
    if let Some(n) = notes { evt = evt.field("notes", json!(n)); }
    if let Some(p) = preservation { evt = evt.field("preservation", p); }
    if let Some(ps) = preservation_supported { evt = evt.field("preservation_supported", json!(ps)); }
    evt.emit_success();
}
