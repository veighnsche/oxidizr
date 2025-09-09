use crate::Result;
use chrono::{Local, SecondsFormat};
use std::env;
use std::fs;
use tracing::{event, Level};

pub const AUDIT_LOG_PATH: &str = "/var/log/oxidizr-arch-audit.log";

/// Best-effort masking of secrets, tokens, and credentials in free-form strings.
fn mask_secrets(s: &str) -> String {
    let mut out = Vec::with_capacity(64);
    for token in s.split_whitespace() {
        // key=value style
        if let Some((k, v)) = token.split_once('=') {
            let kl = k.to_ascii_lowercase();
            if matches!(
                kl.as_str(),
                "token"
                    | "secret"
                    | "password"
                    | "passwd"
                    | "auth"
                    | "authorization"
                    | "bearer"
                    | "access_key"
                    | "secret_key"
                    | "api_key"
                    | "apikey"
            ) {
                out.push(format!("{}=***", k));
                continue;
            }
            // Authorization=Bearer: redact value
            if kl == "authorization" && v.to_ascii_lowercase().starts_with("bearer") {
                out.push(format!("{}=Bearer ***", k));
                continue;
            }
        }
        // Bearer tokens
        let tl = token.to_ascii_lowercase();
        if tl.starts_with("bearer") {
            out.push("Bearer ***".to_string());
            continue;
        }
        out.push(token.to_string());
    }
    out.join(" ")
}

/// Emit a structured audit event to the JSONL sink.
/// Required fields: timestamp (added by formatter), component, event, decision, inputs, outputs, exit_code
pub fn audit_event(
    component: &str,
    event_name: &str,
    decision: &str,
    inputs: &str,
    outputs: &str,
    exit_code: Option<i32>,
) -> Result<()> {
    let inputs = mask_secrets(inputs);
    let outputs = mask_secrets(outputs);
    let timestamp = Local::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    // Correlatable fields
    let run_id = env::var("RUN_ID").unwrap_or_default();
    let container_id = read_container_id();
    let distro = read_distro_id();
    // Canonical level string for JSONL envelope; keep tracing level INFO for routing
    let level_str = match decision.to_ascii_lowercase().as_str() {
        "failure" | "error" => "error",
        _ => "info",
    };
    // Note: exit_code will be rendered as a string by the formatter; presence is what matters.
    event!(
        target: "audit",
        Level::INFO,
        ts = %timestamp,
        component = %"product",
        subsystem = %component,
        level = %level_str,
        run_id = %run_id,
        container_id = %container_id,
        distro = %distro,
        event = %event_name,
        decision = %decision,
        inputs = %inputs,
        outputs = %outputs,
        exit_code = ?exit_code,
        "audit"
    );
    Ok(())
}

/// Convenience wrapper for simple operations (e.g., CREATE_SYMLINK)
pub fn audit_op(operation: &str, target: &str, success: bool) -> Result<()> {
    audit_event(
        "operation",
        operation,
        if success { "success" } else { "failure" },
        target,
        "",
        None,
    )
}

fn read_container_id() -> String {
    // Docker typically sets /etc/hostname to the short container ID
    if let Ok(s) = fs::read_to_string("/etc/hostname") {
        let id = s.trim();
        if !id.is_empty() {
            return id.to_string();
        }
    }
    "".to_string()
}

fn read_distro_id() -> String {
    if let Ok(txt) = fs::read_to_string("/etc/os-release") {
        for line in txt.lines() {
            if let Some(rest) = line.strip_prefix("ID=") {
                return rest.trim_matches('"').to_ascii_lowercase();
            }
        }
    }
    "".to_string()
}
