use log::Level;
use serde_json::Value;

pub trait FactsEmitter: std::fmt::Debug {
    fn emit(&self, subsystem: &str, event: &str, decision: &str, fields: Value);
}

pub trait AuditSink {
    fn log(&self, level: Level, msg: &str);
}

#[derive(Default, Debug, Copy, Clone)]
pub struct JsonlSink;

impl FactsEmitter for JsonlSink {
    fn emit(&self, _subsystem: &str, _event: &str, _decision: &str, _fields: Value) {}
}

impl AuditSink for JsonlSink {
    fn log(&self, _level: Level, _msg: &str) {}
}

// Optional: file-backed JSONL sink for production integration.
// Enabled via `--features file-logging`.
#[cfg(feature = "file-logging")]
#[derive(Debug, Clone)]
pub struct FileJsonlSink {
    path: std::path::PathBuf,
}

#[cfg(feature = "file-logging")]
impl FileJsonlSink {
    pub fn new<P: Into<std::path::PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }
    fn write_line(&self, line: &str) {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            use std::io::Write as _;
            let _ = f.write_all(line.as_bytes());
            let _ = f.write_all(b"\n");
        }
    }
}

#[cfg(feature = "file-logging")]
impl FactsEmitter for FileJsonlSink {
    fn emit(&self, subsystem: &str, event: &str, decision: &str, fields: Value) {
        // Merge subsystem/event/decision into the JSON object if it's an object; otherwise, wrap.
        let out = match fields {
            Value::Object(mut m) => {
                m.entry("subsystem".to_string())
                    .or_insert(Value::from(subsystem));
                m.entry("event".to_string()).or_insert(Value::from(event));
                m.entry("decision".to_string())
                    .or_insert(Value::from(decision));
                Value::Object(m)
            }
            other @ (Value::Null
            | Value::Bool(_)
            | Value::Number(_)
            | Value::String(_)
            | Value::Array(_)) => serde_json::json!({
                "subsystem": subsystem,
                "event": event,
                "decision": decision,
                "fields": other,
            }),
        };
        if let Ok(line) = serde_json::to_string(&out) {
            self.write_line(&line);
        }
    }
}

#[cfg(feature = "file-logging")]
impl AuditSink for FileJsonlSink {
    fn log(&self, level: Level, msg: &str) {
        let out = serde_json::json!({
            "subsystem": "switchyard",
            "event": "audit",
            "decision": "info",
            "level": format!("{}", level),
            "message": msg,
        });
        if let Ok(line) = serde_json::to_string(&out) {
            self.write_line(&line);
        }
    }
}
