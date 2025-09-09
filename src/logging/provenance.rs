use crate::Result;
use super::audit::{audit_event, audit_op};

/// Structured provenance logger shim that emits tracing-backed audit events.
///
/// Kept for backward compatibility; routes all calls to the tracing audit sink.
pub struct ProvenanceLogger;

impl Default for ProvenanceLogger {
    fn default() -> Self {
        Self
    }
}

impl ProvenanceLogger {
    pub fn new() -> Self {
        Self
    }

    /// Log a structured provenance entry (JSONL via tracing layer)
    /// Fields: timestamp, component, event, decision, inputs, outputs, exit_code
    pub fn log(
        &self,
        component: &str,
        event: &str,
        decision: &str,
        inputs: &str,
        outputs: &str,
        exit_code: Option<i32>,
    ) -> Result<()> {
        audit_event(component, event, decision, inputs, outputs, exit_code)
    }

    /// Log a simple operation with success/failure status
    pub fn log_operation(&self, operation: &str, target: &str, success: bool) -> Result<()> {
        audit_op(operation, target, success)
    }
}

// Global provenance logger instance
lazy_static::lazy_static! {
    pub static ref PROVENANCE: ProvenanceLogger = ProvenanceLogger::new();
}
