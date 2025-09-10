//! Adapters wiring switchyard traits to the product's logging and audit sinks.

use super::{AuditSink, FactsEmitter};
use crate::logging::{audit_event_fields, AuditFields};

/// Facts emitter that forwards to the product structured audit JSONL helper.
pub struct ProductFacts;

impl FactsEmitter for ProductFacts {
    fn emit(&self, subsystem: &str, event: &str, decision: &str, fields: serde_json::Value) {
        // Map generic JSON value into AuditFields best-effort; keep extensible.
        let mut af = AuditFields::default();
        if let Some(obj) = fields.as_object() {
            if let Some(stage) = obj.get("stage").and_then(|v| v.as_str()) { af.stage = Some(stage.to_string()); }
            if let Some(suite) = obj.get("suite").and_then(|v| v.as_str()) { af.suite = Some(suite.to_string()); }
            if let Some(cmd) = obj.get("cmd").and_then(|v| v.as_str()) { af.cmd = Some(cmd.to_string()); }
            if let Some(rc) = obj.get("rc").and_then(|v| v.as_i64()) { af.rc = Some(rc as i32); }
            if let Some(d) = obj.get("duration_ms").and_then(|v| v.as_u64()) { af.duration_ms = Some(d); }
            if let Some(t) = obj.get("target").and_then(|v| v.as_str()) { af.target = Some(t.to_string()); }
            if let Some(s) = obj.get("source").and_then(|v| v.as_str()) { af.source = Some(s.to_string()); }
            if let Some(b) = obj.get("backup_path").and_then(|v| v.as_str()) { af.backup_path = Some(b.to_string()); }
            if let Some(arts) = obj.get("artifacts").and_then(|v| v.as_array()) {
                let list: Vec<String> = arts.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect();
                if !list.is_empty() { af.artifacts = Some(list); }
            }
        }
        let _ = audit_event_fields(subsystem, event, decision, &af);
    }
}

/// Audit sink adapter: forward to tracing human logs.
pub struct ProductAudit;

impl AuditSink for ProductAudit {
    fn log(&self, level: log::Level, msg: &str) {
        match level {
            log::Level::Error => tracing::error!("{}", msg),
            log::Level::Warn => tracing::warn!("{}", msg),
            log::Level::Info => tracing::info!("{}", msg),
            log::Level::Debug => tracing::debug!("{}", msg),
            log::Level::Trace => tracing::trace!("{}", msg),
        }
    }
}
