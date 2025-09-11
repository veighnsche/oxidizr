use crate::logging::TS_ZERO;
use crate::logging::FactsEmitter;
use serde_json::json;

pub const SCHEMA_VERSION: i64 = 1;

#[derive(Clone, Debug, Default)]
pub struct AuditMode {
    pub dry_run: bool,
    pub redact: bool,
}

pub struct AuditCtx<'a> {
    pub facts: &'a dyn FactsEmitter,
    pub plan_id: String,
    pub ts: String,
    pub mode: AuditMode,
}

impl<'a> AuditCtx<'a> {
    pub fn new(facts: &'a dyn FactsEmitter, plan_id: String, ts: String, mode: AuditMode) -> Self {
        Self { facts, plan_id, ts, mode }
    }
}

pub fn emit_plan_fact(ctx: &AuditCtx, action_id: &str, path: Option<&str>) {
    let fields = json!({
        "schema_version": SCHEMA_VERSION,
        "ts": TS_ZERO,
        "plan_id": ctx.plan_id,
        "stage": "plan",
        "decision": "success",
        "action_id": action_id,
        "path": path,
    });
    ctx.facts.emit("switchyard", "plan", "success", fields);
}

pub fn emit_preflight_fact(
    ctx: &AuditCtx,
    action_id: &str,
    path: Option<&str>,
    current_kind: &str,
    planned_kind: &str,
) {
    let fields = json!({
        "schema_version": SCHEMA_VERSION,
        "ts": TS_ZERO,
        "plan_id": ctx.plan_id,
        "stage": "preflight",
        "decision": "success",
        "action_id": action_id,
        "path": path,
        "current_kind": current_kind,
        "planned_kind": planned_kind,
    });
    ctx.facts.emit("switchyard", "preflight", "success", fields);
}

pub fn emit_apply_attempt(ctx: &AuditCtx, decision: &str, extra: serde_json::Value) {
    let mut fields = json!({
        "schema_version": SCHEMA_VERSION,
        "ts": ctx.ts,
        "plan_id": ctx.plan_id,
        "stage": "apply.attempt",
        "decision": decision,
        "path": "",
    });
    if let Some(obj) = fields.as_object_mut() {
        for (k, v) in extra.as_object().unwrap_or(&serde_json::Map::new()).iter() {
            obj.insert(k.clone(), v.clone());
        }
    }
    ctx.facts.emit("switchyard", "apply.attempt", decision, fields);
}

pub fn emit_apply_result(ctx: &AuditCtx, decision: &str, extra: serde_json::Value) {
    let mut fields = json!({
        "schema_version": SCHEMA_VERSION,
        "ts": ctx.ts,
        "plan_id": ctx.plan_id,
        "stage": "apply.result",
        "decision": decision,
        "path": "",
    });
    if let Some(obj) = fields.as_object_mut() {
        if let Some(eobj) = extra.as_object() {
            for (k, v) in eobj.iter() {
                obj.insert(k.clone(), v.clone());
            }
        }
    }
    ctx.facts.emit("switchyard", "apply.result", decision, fields);
}

pub fn emit_summary(ctx: &AuditCtx, stage: &str, decision: &str) {
    let fields = json!({
        "schema_version": SCHEMA_VERSION,
        "ts": ctx.ts,
        "plan_id": ctx.plan_id,
        "stage": stage,
        "decision": decision,
        "path": "",
    });
    ctx.facts.emit("switchyard", stage, decision, fields);
}

pub fn emit_rollback_step(ctx: &AuditCtx, decision: &str, path: &str) {
    let fields = json!({
        "schema_version": SCHEMA_VERSION,
        "ts": ctx.ts,
        "plan_id": ctx.plan_id,
        "stage": "rollback",
        "decision": decision,
        "path": path,
    });
    ctx.facts.emit("switchyard", "rollback", decision, fields);
}
