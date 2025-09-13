//! Shared test helpers for the switchyard crate integration tests.

use log::Level;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use switchyard::logging::{AuditSink, FactsEmitter};

/// A simple in-memory emitter to capture facts during tests.
#[derive(Clone, Default, Debug)]
pub struct TestEmitter {
    pub events: Arc<Mutex<Vec<(String, String, String, Value)>>>,
}

impl FactsEmitter for TestEmitter {
    fn emit(&self, subsystem: &str, event: &str, decision: &str, fields: Value) {
        self.events
            .lock()
            .unwrap()
            .push((subsystem.into(), event.into(), decision.into(), fields));
    }
}

/// A no-op audit sink for tests.
#[derive(Clone, Default)]
pub struct TestAudit;

impl AuditSink for TestAudit {
    fn log(&self, _level: Level, _msg: &str) {}
}

/// Create a temporary root directory suitable for building SafePaths.
pub fn with_temp_root() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}
