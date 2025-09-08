use crate::Result;
use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

/// Structured provenance logger for command execution and decisions
pub struct ProvenanceLogger {
    log_path: String,
}

impl Default for ProvenanceLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl ProvenanceLogger {
    pub fn new() -> Self {
        Self {
            log_path: "/var/log/oxidizr-arch-audit.log".to_string(),
        }
    }

    /// Log a structured provenance entry (JSONL format)
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
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        
        // Escape special characters for JSON
        fn esc(s: &str) -> String {
            s.replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
        }
        
        let json = format!(
            "{{\"timestamp\":\"{}\",\"component\":\"{}\",\"event\":\"{}\",\"decision\":\"{}\",\"inputs\":\"{}\",\"outputs\":\"{}\",\"exit_code\":{}}}\n",
            esc(&timestamp),
            esc(component),
            esc(event),
            esc(decision),
            esc(inputs),
            esc(outputs),
            exit_code.map(|c| c.to_string()).unwrap_or_else(|| "null".into())
        );

        // Try system path first, fallback to user log on error
        if self.write_to_file(&self.log_path, &json).is_err() {
            let user_log = format!(
                "{}/.oxidizr-arch-audit.log",
                std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
            );
            self.write_to_file(&user_log, &json)?;
        }
        Ok(())
    }

    /// Log a simple operation with success/failure status
    pub fn log_operation(&self, operation: &str, target: &str, success: bool) -> Result<()> {
        self.log(
            "operation",
            operation,
            if success { "success" } else { "failure" },
            target,
            "",
            None,
        )
    }

    fn write_to_file(&self, path: &str, content: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(Path::new(path))?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

// Global provenance logger instance
lazy_static::lazy_static! {
    pub static ref PROVENANCE: ProvenanceLogger = ProvenanceLogger::new();
}
