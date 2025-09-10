use std::path::PathBuf;
use cucumber::World;

#[derive(Debug, Default, World)]
pub struct TestWorld {
    // Root temp dir per scenario; later we can mount fake FS or sandbox
    pub work: PathBuf,
    // Optional: collected audit events for assertions
    pub audit_events: Vec<serde_json::Value>,
    // Optional: capture exit codes as per SPEC/error_codes.toml
    pub last_exit_code: Option<i32>,
}
