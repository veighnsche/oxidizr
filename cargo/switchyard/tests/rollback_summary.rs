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
fn rollback_summary_emitted_when_action_failure_triggers_rollback() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.allow_unlocked_commit = true; // no lock required
    policy.force_untrusted_source = true; // allow non-root-owned sources in test

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    // First pair: will succeed
    let src1 = root.join("bin/new1");
    let tgt1 = root.join("usr/bin/app1");
    std::fs::create_dir_all(src1.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt1.parent().unwrap()).unwrap();
    std::fs::write(&src1, b"n1").unwrap();
    std::fs::write(root.join("usr/bin/app1"), b"o1").unwrap();

    // Second pair: parent dir without write perms to force failure during swap
    let src2 = root.join("bin/new2");
    let denied_dir = root.join("usr/denied");
    let tgt2 = denied_dir.join("app2");
    std::fs::create_dir_all(src2.parent().unwrap()).unwrap();
    std::fs::create_dir_all(&denied_dir).unwrap();
    std::fs::write(&src2, b"n2").unwrap();
    // Drop write bit on parent dir so unlink/rename fails
    let mut perms = std::fs::metadata(&denied_dir).unwrap().permissions();
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o555);
    }
    std::fs::set_permissions(&denied_dir, perms).unwrap();

    let s1 = SafePath::from_rooted(root, &src1).unwrap();
    let t1 = SafePath::from_rooted(root, &tgt1).unwrap();
    let s2 = SafePath::from_rooted(root, &src2).unwrap();
    let t2 = SafePath::from_rooted(root, &tgt2).unwrap();

    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: s1, target: t1 }, LinkRequest { source: s2, target: t2 }], restore: vec![] });

    let report = api.apply(&plan, ApplyMode::Commit).unwrap();
    assert!(report.rolled_back, "expected rollback on failure");

    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    assert!(redacted.iter().any(|e| e.get("stage") == Some(&Value::from("rollback.summary"))),
            "expected rollback.summary event");
}
