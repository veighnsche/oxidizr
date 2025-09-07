use crate::error::Result;
use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

/// Audit logger for security-sensitive operations
pub struct AuditLogger {
    log_path: String,
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
        if let Err(_) = self.write_to_file(&self.log_path, &log_entry) {
            let user_log = format!(
                "{}/.oxidizr-arch-audit.log",
                std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
            );
            self.write_to_file(&user_log, &log_entry)?;
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
