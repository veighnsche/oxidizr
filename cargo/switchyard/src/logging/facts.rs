use log::Level;
use serde_json::Value;

pub trait FactsEmitter {
    fn emit(&self, subsystem: &str, event: &str, decision: &str, fields: Value);
}

pub trait AuditSink {
    fn log(&self, level: Level, msg: &str);
}

#[derive(Default)]
pub struct JsonlSink;

impl FactsEmitter for JsonlSink {
    fn emit(&self, _subsystem: &str, _event: &str, _decision: &str, _fields: Value) {}
}

impl AuditSink for JsonlSink {
    fn log(&self, _level: Level, _msg: &str) {}
}
