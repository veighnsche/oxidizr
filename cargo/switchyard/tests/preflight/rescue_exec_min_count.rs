//! E2E-PREFLIGHT-005 — Exec check with min_count=100 → STOP (REQ-RC2)

use serial_test::serial;
use switchyard::logging::{FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

#[derive(Default, Clone, Debug)]
struct TestEmitter;
impl FactsEmitter for TestEmitter {
    fn emit(&self, _subsystem: &str, _event: &str, _decision: &str, _fields: serde_json::Value) {}
}

#[test]
#[serial]
fn preflight_stops_when_exec_check_and_min_count_huge() {
    // Force an empty PATH so exec checks fail; ensure rescue.require=true and min_count=100
    let _guard = EnvGuard::new();
    std::env::set_var("PATH", "");

    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.rescue.require = true;
    policy.rescue.exec_check = true;
    policy.rescue.min_count = 100;
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;

    let api = switchyard::Switchyard::new(facts, audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"n").unwrap();
    std::fs::write(root.join("usr/bin/app"), b"o").unwrap();

    let s = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(root, &root.join("usr/bin/app")).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t,
        }],
        restore: vec![],
    });

    let pf = api.preflight(&plan).unwrap();
    assert!(
        !pf.ok,
        "preflight should STOP when exec_check enabled and min_count huge"
    );
    let stops = pf.stops.join("\n");
    assert!(
        stops.to_lowercase().contains("rescue"),
        "expected rescue-related stop message: {}",
        stops
    );
}

struct EnvGuard(Option<std::ffi::OsString>);
impl EnvGuard {
    fn new() -> Self {
        let prev = std::env::var_os("PATH");
        EnvGuard(prev)
    }
}
impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(ref v) = self.0 {
            std::env::set_var("PATH", v);
        } else {
            std::env::remove_var("PATH");
        }
    }
}
