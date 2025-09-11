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
    let fields = json!({
        "ts": TS_ZERO,
        "stage": "plan",
        "decision": "success",
        "action_id": action_id,
        "path": path,
    });
    redact_and_emit(ctx, "switchyard", "plan", "success", fields);
}

pub(crate) fn emit_preflight_fact(
    ctx: &AuditCtx,
    action_id: &str,
    path: Option<&str>,
    current_kind: &str,
    planned_kind: &str,
) {
    let fields = json!({
        "ts": TS_ZERO,
        "stage": "preflight",
        "decision": "success",
        "action_id": action_id,
        "path": path,
        "current_kind": current_kind,
        "planned_kind": planned_kind,
    });
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
    redact_and_emit(ctx, "switchyard", "apply.attempt", decision, fields);
}

pub(crate) fn emit_apply_result(ctx: &AuditCtx, decision: &str, mut extra: Value) {
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
    redact_and_emit(ctx, "switchyard", "apply.result", decision, fields);
}

pub(crate) fn emit_summary(ctx: &AuditCtx, stage: &str, decision: &str) {
    let fields = json!({
        "stage": stage,
        "decision": decision,
    });
    redact_and_emit(ctx, "switchyard", stage, decision, fields);
}

pub(crate) fn emit_rollback_step(ctx: &AuditCtx, decision: &str, path: &str) {
    let fields = json!({
        "stage": "rollback",
        "decision": decision,
        "path": path,
    });
    redact_and_emit(ctx, "switchyard", "rollback", decision, fields);
}

// Optional helper to ensure a provenance object is present; callers may extend as needed.
pub(crate) fn ensure_provenance(extra: &mut Value) {
    if let Some(obj) = extra.as_object_mut() {
        obj.entry("provenance").or_insert(json!({
            "helper": "",
            "env_sanitized": true
        }));
    }
}
