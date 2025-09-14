//! E2E-APPLY-017 — Ownership strict with oracle present (REQ-S4, REQ-O7)
//! Assert provenance present in facts.

use serde_json::Value;
use switchyard::logging::{redact_event, FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[derive(Default, Clone, Debug)]
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
#[ignore = "see BUGS.md:apply-ownership-strict-with-oracle"]
fn e2e_apply_017_ownership_strict_with_oracle_present() {
    // REQ-S4, REQ-O7, E2E-APPLY-017 (P1)
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.risks.ownership_strict = true; // Enable strict ownership policy
    policy.governance.allow_unlocked_commit = true; // avoid lock manager requirement
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted; // avoid gating on temp files
    policy.apply.override_preflight = true; // skip preflight checks

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_ownership_oracle(Box::new(switchyard::adapters::FsOwnershipOracle::default()));

    // Layout under temp root
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"n").unwrap();
    std::fs::write(&tgt, b"o").unwrap();

    let s = SafePath::from_rooted(root, &src).unwrap();
    let t = SafePath::from_rooted(root, &tgt).unwrap();
    let input = PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t,
        }],
        restore: vec![],
    };

    let plan = api.plan(input);

    // Apply should succeed with ownership oracle present
    let _report = api.apply(&plan, ApplyMode::Commit).unwrap();

    // Check that we got the appropriate apply events with provenance
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();

    // Should have an apply.result success event with provenance information
    let provenance_event = redacted.iter().find(|e| {
        e.get("stage") == Some(&Value::from("apply.result"))
            && e.get("decision") == Some(&Value::from("success"))
            && e.get("provenance").is_some()
    });

    assert!(
        provenance_event.is_some(),
        "expected apply.result success with provenance information"
    );

    // Check that provenance includes uid/gid/pkg fields
    if let Some(event) = provenance_event {
        let provenance = event.get("provenance").unwrap();
        assert!(
            provenance.get("uid").is_some(),
            "provenance should include uid field"
        );
        assert!(
            provenance.get("gid").is_some(),
            "provenance should include gid field"
        );
        assert!(
            provenance.get("pkg").is_some(),
            "provenance should include pkg field"
        );
    }
}
