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
fn preflight_stops_when_rescue_required_and_unavailable() {
    let _guard = env_guard();
    // Force no tools on PATH and override verification to fail
    set_path("");
    std::env::set_var("SWITCHYARD_FORCE_RESCUE_OK", "0");

    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.rescue.require = true;
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;

    let api = switchyard::Switchyard::new(facts, audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"new").unwrap();
    std::fs::write(root.join("usr/bin/ls"), b"old").unwrap();

    let s = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(root, &root.join("usr/bin/ls")).unwrap();
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
        "preflight should fail-closed when rescue required and unavailable"
    );
    let msg = pf.stops.join("\n");
    assert!(
        msg.contains("rescue profile unavailable"),
        "expected rescue stop in preflight stops: {}",
        msg
    );
}

#[test]
#[serial]
fn preflight_succeeds_when_rescue_required_and_available() {
    let _guard = env_guard();
    // Create a temp dir with a fake busybox file and set PATH to it, also force verification to pass
    let tdpath = {
        let td = tempfile::tempdir().unwrap();
        let p = td.path().to_path_buf();
        std::fs::write(p.join("busybox"), b"").unwrap();
        // keep dir alive by leaking; acceptable in tests
        std::mem::forget(td);
        p
    };
    set_path(tdpath.to_str().unwrap());
    std::env::set_var("SWITCHYARD_FORCE_RESCUE_OK", "1");

    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.rescue.require = true;
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;

    let api = switchyard::Switchyard::new(facts, audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"new").unwrap();
    std::fs::write(root.join("usr/bin/ls"), b"old").unwrap();

    let s = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(root, &root.join("usr/bin/ls")).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t,
        }],
        restore: vec![],
    });

    let pf = api.preflight(&plan).unwrap();
    assert!(pf.ok, "preflight should succeed when rescue available");
}

fn env_guard() -> impl Drop {
    let prev = std::env::var_os("PATH");
    struct D(Option<std::ffi::OsString>);
    impl Drop for D {
        fn drop(&mut self) {
            if let Some(ref v) = self.0 {
                std::env::set_var("PATH", v);
            } else {
                std::env::remove_var("PATH");
            }
            std::env::remove_var("SWITCHYARD_FORCE_RESCUE_OK");
        }
    }
    D(prev)
}

fn set_path(s: &str) {
    std::env::set_var("PATH", s);
}
