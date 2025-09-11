use crate::logging::{redact_event, FactsEmitter, TS_ZERO};
use serde_json::{json, Value};

pub(crate) const SCHEMA_VERSION: i64 = 1;

#[derive(Clone, Debug, Default)]
pub(crate) struct AuditMode {
    pub dry_run: bool,
    pub redact: bool,
}

pub(crate) struct AuditCtx<'a> {
    pub facts: &'a dyn FactsEmitter,
    pub plan_id: String,
    pub ts: String,
    pub mode: AuditMode,
}

impl<'a> AuditCtx<'a> {
    pub(crate) fn new(
        facts: &'a dyn FactsEmitter,
        plan_id: String,
        ts: String,
        mode: AuditMode,
    ) -> Self {
        Self { facts, plan_id, ts, mode }
    }
}

fn redact_and_emit(ctx: &AuditCtx, subsystem: &str, event: &str, decision: &str, mut fields: Value) {
    // Ensure minimal envelope fields
    if let Some(obj) = fields.as_object_mut() {
        obj.entry("schema_version").or_insert(json!(SCHEMA_VERSION));
        obj.entry("ts").or_insert(json!(ctx.ts));
        obj.entry("plan_id").or_insert(json!(ctx.plan_id));
        obj.entry("path").or_insert(json!(""));
    }
    // Apply redaction policy in dry-run or when requested
    let out = if ctx.mode.redact { redact_event(fields) } else { fields };
    ctx.facts.emit(subsystem, event, decision, out);
}

pub(crate) fn emit_plan_fact(ctx: &AuditCtx, action_id: &str, path: Option<&str>) {
    let mut fields = json!({
        "ts": TS_ZERO,
        "stage": "plan",
        "decision": "success",
        "action_id": action_id,
        "path": path,
    });
    ensure_provenance(&mut fields);
    redact_and_emit(ctx, "switchyard", "plan", "success", fields);
}

pub(crate) fn emit_preflight_fact(
    ctx: &AuditCtx,
    action_id: &str,
    path: Option<&str>,
    current_kind: &str,
    planned_kind: &str,
) {
    let mut fields = json!({
        "ts": TS_ZERO,
        "stage": "preflight",
        "decision": "success",
        "action_id": action_id,
        "path": path,
        "current_kind": current_kind,
        "planned_kind": planned_kind,
    });
    ensure_provenance(&mut fields);
    redact_and_emit(ctx, "switchyard", "preflight", "success", fields);
}

pub(crate) fn emit_preflight_fact_ext(
    ctx: &AuditCtx,
    action_id: &str,
    path: Option<String>,
    current_kind: &str,
    planned_kind: &str,
    policy_ok: Option<bool>,
    provenance: Option<Value>,
    notes: Option<Vec<String>>,
    preservation: Option<Value>,
    preservation_supported: Option<bool>,
){
    let mut fields = json!({
        "ts": TS_ZERO,
        "stage": "preflight",
        "decision": "success",
        "action_id": action_id,
        "path": path,
        "current_kind": current_kind,
        "planned_kind": planned_kind,
    });
    if let Some(obj) = fields.as_object_mut() {
        if let Some(ok) = policy_ok { obj.insert("policy_ok".into(), json!(ok)); }
        if let Some(p) = provenance { obj.insert("provenance".into(), p); }
        if let Some(n) = notes { obj.insert("notes".into(), json!(n)); }
        if let Some(p) = preservation { obj.insert("preservation".into(), p); }
        if let Some(s) = preservation_supported { obj.insert("preservation_supported".into(), json!(s)); }
    }
    ensure_provenance(&mut fields);
    redact_and_emit(ctx, "switchyard", "preflight", "success", fields);
}

pub(crate) fn emit_apply_attempt(ctx: &AuditCtx, decision: &str, extra: Value) {
    let mut fields = json!({
        "stage": "apply.attempt",
        "decision": decision,
    });
    if let Some(obj) = fields.as_object_mut() {
        for (k, v) in extra.as_object().unwrap_or(&serde_json::Map::new()).iter() {
            obj.insert(k.clone(), v.clone());
        }
    }
    ensure_provenance(&mut fields);
    redact_and_emit(ctx, "switchyard", "apply.attempt", decision, fields);
}

pub(crate) fn emit_apply_result(ctx: &AuditCtx, decision: &str, extra: Value) {
    let mut fields = json!({
        "stage": "apply.result",
        "decision": decision,
    });
    if let Some(obj) = fields.as_object_mut() {
        if let Some(eobj) = extra.as_object() {
            for (k, v) in eobj.iter() {
                obj.insert(k.clone(), v.clone());
            }
        }
    }
    ensure_provenance(&mut fields);
    redact_and_emit(ctx, "switchyard", "apply.result", decision, fields);
}

pub(crate) fn emit_summary(ctx: &AuditCtx, stage: &str, decision: &str) {
    let mut fields = json!({
        "stage": stage,
        "decision": decision,
    });
    ensure_provenance(&mut fields);
    redact_and_emit(ctx, "switchyard", stage, decision, fields);
}

pub(crate) fn emit_summary_extra(ctx: &AuditCtx, stage: &str, decision: &str, extra: Value) {
    let mut fields = json!({
        "stage": stage,
        "decision": decision,
    });
    if let Some(obj) = fields.as_object_mut() {
        if let Some(eobj) = extra.as_object() {
            for (k, v) in eobj.iter() {
                obj.insert(k.clone(), v.clone());
            }
        }
    }
    ensure_provenance(&mut fields);
    redact_and_emit(ctx, "switchyard", stage, decision, fields);
}

pub(crate) fn emit_rollback_step(ctx: &AuditCtx, decision: &str, path: &str) {
    let mut fields = json!({
        "stage": "rollback",
        "decision": decision,
        "path": path,
    });
    ensure_provenance(&mut fields);
    redact_and_emit(ctx, "switchyard", "rollback", decision, fields);
}

// Optional helper to ensure a provenance object is present; callers may extend as needed.
pub(crate) fn ensure_provenance(extra: &mut Value) {
    if let Some(obj) = extra.as_object_mut() {
        let prov = obj
            .entry("provenance")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();
        // Only enforce env_sanitized presence by default; origin/helper are adapter-provided.
        prov.entry("env_sanitized").or_insert(json!(true));
    }
}
