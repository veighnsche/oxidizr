use std::path::PathBuf;
use cucumber::World;
use tempfile::TempDir;

#[derive(Debug, World)]
pub struct TestWorld {
    // Root temp dir per scenario; later we can mount fake FS or sandbox
    pub work: PathBuf,
    // Optional: collected audit events for assertions
    pub audit_events: Vec<serde_json::Value>,
    // Optional: capture exit codes as per SPEC/error_codes.toml
    pub last_exit_code: Option<i32>,
    // Keep tempdir handle to auto-clean after scenario
    _tmp: TempDir,
}

impl Default for TestWorld {
    fn default() -> Self {
        let tmp = TempDir::new().expect("create scenario tempdir");
        let work = tmp.path().to_path_buf();
        Self { work, audit_events: Vec::new(), last_exit_code: None, _tmp: tmp }
    }
}
