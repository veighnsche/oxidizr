use serde_json::{json, Value};
use uuid::Uuid;

use crate::logging::{AuditSink, FactsEmitter};
use crate::types::{Action, Plan};

use crate::logging::audit::{emit_preflight_fact_ext, AuditCtx};

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
    rows.push(row);

    // Emit fact
    emit_preflight_fact_ext(
        ctx,
        &aid.to_string(),
        Some(path),
        &current_kind,
        planned_kind,
        policy_ok,
        provenance,
        notes,
        preservation,
        preservation_supported,
    );
}
