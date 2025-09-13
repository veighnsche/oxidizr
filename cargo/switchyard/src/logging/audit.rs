/// replace this file with `StageLogger` facade — see `zrefactor/logging_audit_refactor.INSTRUCTIONS.md`
// Audit helpers that emit Minimal Facts v1 across Switchyard stages.
//
// Side-effects:
// - Emits JSON facts via `FactsEmitter` for the following stages:
//   - `plan`, `preflight` (per-action rows and summary), `apply.attempt`, `apply.result`, and `rollback` steps.
// - Ensures a minimal envelope is present on every fact: `schema_version`, `ts`, `plan_id`, `path`.
// - Applies redaction in dry-run to zero timestamps and drop volatile fields.
//
// See `SPEC/SPEC.md` for field semantics and Minimal Facts v1 schema.
use crate::logging::{redact_event, FactsEmitter};
use serde_json::{json, Map, Value};
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub(crate) const SCHEMA_VERSION: i64 = 2;

#[derive(Clone, Debug, Default)]
pub(crate) struct AuditMode {
    pub dry_run: bool,
    pub redact: bool,
}

#[derive(Debug)]
pub(crate) struct AuditCtx<'a> {
    pub facts: &'a dyn FactsEmitter,
    pub plan_id: String,
    pub run_id: String,
    pub ts: String,
    pub mode: AuditMode,
    pub seq: Cell<u64>,
}

impl<'a> AuditCtx<'a> {
    pub(crate) fn new(
        facts: &'a dyn FactsEmitter,
        plan_id: String,
        run_id: String,
        ts: String,
        mode: AuditMode,
    ) -> Self {
        Self {
            facts,
            plan_id,
            run_id,
            ts,
            mode,
            seq: Cell::new(0),
        }
    }
}

/// Stage for typed audit emission.
#[derive(Clone, Copy, Debug)]
pub enum Stage {
    Plan,
    Preflight,
    PreflightSummary,
    ApplyAttempt,
    ApplyResult,
    Rollback,
    RollbackSummary,
    PruneResult,
}

impl Stage {
    const fn as_event(self) -> &'static str {
        match self {
            Stage::Plan => "plan",
            Stage::Preflight => "preflight",
            Stage::PreflightSummary => "preflight.summary",
            Stage::ApplyAttempt => "apply.attempt",
            Stage::ApplyResult => "apply.result",
            Stage::Rollback => "rollback",
            Stage::RollbackSummary => "rollback.summary",
            Stage::PruneResult => "prune.result",
        }
    }
}

/// Decision severity for audit events.
#[derive(Clone, Copy, Debug)]
pub enum Decision {
    Success,
    Failure,
    Warn,
}

impl Decision {
    const fn as_str(self) -> &'static str {
        match self {
            Decision::Success => "success",
            Decision::Failure => "failure",
            Decision::Warn => "warn",
        }
    }
}

/// Builder facade over audit emission with centralized envelope+redaction.
#[derive(Debug)]
pub struct StageLogger<'a> {
    ctx: &'a AuditCtx<'a>,
}

impl<'a> StageLogger<'a> {
    pub(crate) const fn new(ctx: &'a AuditCtx<'a>) -> Self {
        Self { ctx }
    }

    #[must_use]
    pub fn plan(&'a self) -> EventBuilder<'a> {
        EventBuilder::new(self.ctx, Stage::Plan)
    }
    #[must_use]
    pub fn preflight(&'a self) -> EventBuilder<'a> {
        EventBuilder::new(self.ctx, Stage::Preflight)
    }
    #[must_use]
    pub fn preflight_summary(&'a self) -> EventBuilder<'a> {
        EventBuilder::new(self.ctx, Stage::PreflightSummary)
    }
    #[must_use]
    pub fn apply_attempt(&'a self) -> EventBuilder<'a> {
        EventBuilder::new(self.ctx, Stage::ApplyAttempt)
    }
    #[must_use]
    pub fn apply_result(&'a self) -> EventBuilder<'a> {
        EventBuilder::new(self.ctx, Stage::ApplyResult)
    }
    #[must_use]
    pub fn rollback(&'a self) -> EventBuilder<'a> {
        EventBuilder::new(self.ctx, Stage::Rollback)
    }
    #[must_use]
    pub fn rollback_summary(&'a self) -> EventBuilder<'a> {
        EventBuilder::new(self.ctx, Stage::RollbackSummary)
    }
    #[must_use]
    pub fn prune_result(&'a self) -> EventBuilder<'a> {
        EventBuilder::new(self.ctx, Stage::PruneResult)
    }
}

#[derive(Debug)]
pub struct EventBuilder<'a> {
    ctx: &'a AuditCtx<'a>,
    stage: Stage,
    fields: Map<String, Value>,
}

impl<'a> EventBuilder<'a> {
    fn new(ctx: &'a AuditCtx<'a>, stage: Stage) -> Self {
        let mut fields = Map::new();
        fields.insert("stage".to_string(), json!(stage.as_event()));
        Self { ctx, stage, fields }
    }

    #[must_use]
    pub fn action(mut self, action_id: impl Into<String>) -> Self {
        self.fields
            .insert("action_id".into(), json!(action_id.into()));
        self
    }

    /// Thin wrapper for `.action(...)` to improve readability at call sites.
    #[must_use]
    pub fn action_id(self, aid: impl Into<String>) -> Self {
        self.action(aid)
    }

    #[must_use]
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.fields.insert("path".into(), json!(path.into()));
        self
    }

    /// Attach a nested perf object with hash/backup/swap timings in milliseconds.
    #[must_use]
    pub fn perf(mut self, hash_ms: u64, backup_ms: u64, swap_ms: u64) -> Self {
        self.fields.insert(
            "perf".to_string(),
            json!({
                "hash_ms": hash_ms,
                "backup_ms": backup_ms,
                "swap_ms": swap_ms,
            }),
        );
        self
    }

    /// Set a stable error identifier as defined in `crate::api::errors`.
    #[must_use]
    pub fn error_id(mut self, id: crate::api::errors::ErrorId) -> Self {
        self.fields.insert(
            "error_id".to_string(),
            json!(crate::api::errors::id_str(id)),
        );
        self
    }

    /// Set an exit code derived from the given error id.
    #[must_use]
    pub fn exit_code_for(mut self, id: crate::api::errors::ErrorId) -> Self {
        self.fields.insert(
            "exit_code".to_string(),
            json!(crate::api::errors::exit_code_for(id)),
        );
        self
    }

    #[must_use]
    pub fn field(mut self, key: &str, value: Value) -> Self {
        self.fields.insert(key.to_string(), value);
        self
    }

    #[must_use]
    pub fn merge(mut self, extra: &Value) -> Self {
        if let Some(obj) = extra.as_object() {
            for (k, v) in obj {
                self.fields.insert(k.clone(), v.clone());
            }
        }
        self
    }

    pub fn emit(self, decision: Decision) {
        let mut fields = Value::Object(self.fields);
        // Ensure provenance object present by default
        ensure_provenance(&mut fields);
        if let Some(obj) = fields.as_object_mut() {
            obj.entry("decision").or_insert(json!(decision.as_str()));
        }
        redact_and_emit(
            self.ctx,
            "switchyard",
            self.stage.as_event(),
            decision.as_str(),
            fields,
        );
    }

    pub fn emit_success(self) {
        self.emit(Decision::Success);
    }
    pub fn emit_failure(self) {
        self.emit(Decision::Failure);
    }
    pub fn emit_warn(self) {
        self.emit(Decision::Warn);
    }
}

fn redact_and_emit(
    ctx: &AuditCtx<'_>,
    subsystem: &str,
    event: &str,
    decision: &str,
    mut fields: Value,
) {
    // Ensure minimal envelope fields
    if let Some(obj) = fields.as_object_mut() {
        obj.entry("schema_version").or_insert(json!(SCHEMA_VERSION));
        obj.entry("ts").or_insert(json!(ctx.ts));
        obj.entry("plan_id").or_insert(json!(ctx.plan_id));
        obj.entry("run_id").or_insert(json!(ctx.run_id));
        obj.entry("event_id").or_insert(json!(new_event_id()));
        obj.entry("switchyard_version")
            .or_insert(json!(env!("CARGO_PKG_VERSION")));
        // Redaction metadata (lightweight)
        obj.entry("redacted").or_insert(json!(ctx.mode.redact));
        obj.entry("redaction")
            .or_insert(json!({"applied": ctx.mode.redact}));

        // Optional envmeta (host/process/actor/build)
        #[cfg(feature = "envmeta")]
        {
            use serde_json::map::Entry;
            // host
            if let Entry::Vacant(e) = obj.entry("host".to_string()) {
                let hostname = std::env::var("HOSTNAME").ok();
                let os = Some(std::env::consts::OS.to_string());
                let arch = Some(std::env::consts::ARCH.to_string());
                // Kernel best-effort: read from /proc/version if present
                let kernel = std::fs::read_to_string("/proc/version")
                    .ok()
                    .and_then(|s| s.split_whitespace().nth(2).map(ToString::to_string));
                e.insert(json!({
                    "hostname": hostname,
                    "os": os,
                    "kernel": kernel,
                    "arch": arch,
                }));
            }
            // process
            if let Entry::Vacant(e) = obj.entry("process".to_string()) {
                let process_id = std::process::id();
                let parent_process_id = rustix::process::getppid().as_raw();
                e.insert(json!({"pid": process_id, "ppid": parent_process_id}));
            }
            // actor (effective ids)
            if let Entry::Vacant(e) = obj.entry("actor".to_string()) {
                let effective_user_id = rustix::process::geteuid().as_raw();
                let effective_group_id = rustix::process::getegid().as_raw();
                e.insert(json!({"euid": effective_user_id, "egid": effective_group_id}));
            }
            // build
            if let Entry::Vacant(e) = obj.entry("build".to_string()) {
                let git_sha = std::env::var("GIT_SHA").ok();
                let rustc = std::env::var("RUSTC_VERSION").ok();
                e.insert(json!({"git_sha": git_sha, "rustc": rustc}));
            }
        }
        // Monotonic per-run sequence
        let cur = ctx.seq.get();
        obj.entry("seq").or_insert(json!(cur));
        ctx.seq.set(cur.saturating_add(1));
        obj.entry("dry_run").or_insert(json!(ctx.mode.dry_run));
    }
    // Apply redaction policy in dry-run or when requested
    let out = if ctx.mode.redact {
        redact_event(fields)
    } else {
        fields
    };
    ctx.facts.emit(subsystem, event, decision, out);
}

fn new_event_id() -> String {
    // Derive a name from (nanos_since_epoch, counter) for uniqueness, then build UUID v5
    static NEXT_ID_COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let c = NEXT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = format!("{nanos}:{c}:event");
    Uuid::new_v5(&Uuid::NAMESPACE_URL, name.as_bytes()).to_string()
}

pub(crate) fn new_run_id() -> String {
    // Similar generation strategy as event_id, but with a different tag
    static NEXT_RUN_COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let c = NEXT_RUN_COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = format!("{nanos}:{c}:run");
    Uuid::new_v5(&Uuid::NAMESPACE_URL, name.as_bytes()).to_string()
}

// Legacy emit_* helpers have been removed; use StageLogger facade exclusively.

// Optional helper to ensure a provenance object is present; callers may extend as needed.
/// Ensure `extra["provenance"]` is an object and contains `env_sanitized: true`.
pub(crate) fn ensure_provenance(extra: &mut Value) {
    if let Some(obj) = extra.as_object_mut() {
        // Get or create the "provenance" field as an object
        let prov = obj
            .entry("provenance")
            .or_insert_with(|| Value::Object(Map::new()));

        // If it existed but wasn't an object, replace it with an empty object
        if !prov.is_object() {
            *prov = Value::Object(Map::new());
        }

        // Now safely insert the key
        if let Value::Object(prov_obj) = prov {
            prov_obj.entry("env_sanitized").or_insert(Value::Bool(true));
        }
    }
}
