use serde_json::Value;
use switchyard::logging::{redact_event, FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[derive(Default, Clone)]
struct TestEmitter {
    events: std::sync::Arc<std::sync::Mutex<Vec<(String, String, String, Value)>>>,
}

impl FactsEmitter for TestEmitter {
    fn emit(&self, subsystem: &str, event: &str, decision: &str, fields: Value) {
        self.events.lock().unwrap().push((
            subsystem.to_string(),
            event.to_string(),
            decision.to_string(),
            fields,
        ));
    }
}

#[test]
fn provenance_present_and_env_sanitized_across_stages_including_rollback() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.allow_degraded_fs = true;
    policy.force_untrusted_source = true;
    policy.allow_unlocked_commit = true; // allow Commit without LockManager

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_ownership_oracle(Box::new(switchyard::adapters::FsOwnershipOracle::default()));

    // Layout with two actions; second will fail to trigger rollback of first
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/sbin")).unwrap();
    std::fs::write(root.join("bin/new1"), b"n1").unwrap();
    std::fs::write(root.join("bin/new2"), b"n2").unwrap();
    std::fs::write(root.join("usr/bin/app1"), b"o1").unwrap();
    std::fs::write(root.join("usr/sbin/app2"), b"o2").unwrap();

    // Make second target parent read-only to force failure during apply
    let sbin_dir = root.join("usr/sbin");
    let mut p = std::fs::metadata(&sbin_dir).unwrap().permissions();
    use std::os::unix::fs::PermissionsExt;
    p.set_mode(0o555);
    std::fs::set_permissions(&sbin_dir, p).unwrap();

    let s1 = SafePath::from_rooted(root, &root.join("bin/new1")).unwrap();
    let t1 = SafePath::from_rooted(root, &root.join("usr/bin/app1")).unwrap();
    let s2 = SafePath::from_rooted(root, &root.join("bin/new2")).unwrap();
    let t2 = SafePath::from_rooted(root, &root.join("usr/sbin/app2")).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![
            LinkRequest {
                source: s1,
                target: t1,
            },
            LinkRequest {
                source: s2,
                target: t2,
            },
        ],
        restore: vec![],
    });

    let _ = api.preflight(&plan).unwrap();
    let _ = api.apply(&plan, ApplyMode::Commit).unwrap();

    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    // Stages of interest
    let stages = [
        "plan",
        "preflight",
        "apply.attempt",
        "apply.result",
        "rollback",
    ];

    for v in &redacted {
        if let Some(stage) = v.get("stage").and_then(|s| s.as_str()) {
            if stages.contains(&stage) {
                let prov = v
                    .get("provenance")
                    .and_then(|p| p.as_object())
                    .expect("provenance object");
                assert_eq!(
                    prov.get("env_sanitized").and_then(|b| b.as_bool()),
                    Some(true),
                    "env_sanitized should be true on stage {}",
                    stage
                );
            }
        }
    }
}
