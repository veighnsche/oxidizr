use crate::error::Result;
use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

/// Audit logger for security-sensitive operations
pub struct AuditLogger {
    log_path: String,
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditLogger {
    pub fn new() -> Self {
        Self {
            log_path: "/var/log/oxidizr-arch-audit.log".to_string(),
        }
    }

    pub fn log_operation(&self, operation: &str, target: &str, success: bool) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        let uid = unsafe { libc::getuid() };
        let status = if success { "SUCCESS" } else { "FAILURE" };

        let log_entry = format!(
            "[{}] USER={} UID={} OPERATION={} TARGET={} STATUS={}\n",
            timestamp, user, uid, operation, target, status
        );

        // Try to write to system log, fall back to user log if permission denied
        if self.write_to_file(&self.log_path, &log_entry).is_err() {
            let user_log = format!(
                "{}/.oxidizr-arch-audit.log",
                std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
            );
            self.write_to_file(&user_log, &log_entry)?;
        }

        Ok(())
    }

    /// Writes a structured provenance entry (JSONL) for executed commands and decisions.
    /// Fields: timestamp, component, event, decision, inputs, outputs, exit_code
    pub fn log_provenance(
        &self,
        component: &str,
        event: &str,
        decision: &str,
        inputs: &str,
        outputs: &str,
        exit_code: Option<i32>,
    ) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
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

    fn write_to_file(&self, path: &str, content: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(Path::new(path))?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

// Global audit logger instance
lazy_static::lazy_static! {
    pub static ref AUDIT: AuditLogger = AuditLogger::new();
}
