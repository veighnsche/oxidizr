//! E2E-PREFLIGHT-002 — Ownership strict without oracle → STOP (REQ-S4)
//! Asserts preflight ok=false, stops mention ownership, and summary_error_ids includes E_OWNERSHIP.

use serde_json::Value;
use switchyard::logging::{redact_event, FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

#[derive(Default, Clone, Debug)]
struct TestEmitter {
    events: std::sync::Arc<std::sync::Mutex<Vec<(String, String, String, Value)>>> ,
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
fn ownership_strict_without_oracle_stops_preflight() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.risks.ownership_strict = true; // strict ownership enabled

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"n").unwrap();
    std::fs::write(root.join("usr/bin/app"), b"o").unwrap();

    let s = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(root, &root.join("usr/bin/app")).unwrap();
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: s, target: t }], restore: vec![] });

    let pf = api.preflight(&plan).unwrap();
    assert!(!pf.ok, "preflight should STOP when strict ownership without oracle");
    let stops = pf.stops.join("\n").to_lowercase();
    assert!(stops.contains("ownership"), "expected ownership mentioned in stops: {}", stops);

    // Inspect redacted events for summary_error_ids chain
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();
    let summary = redacted.iter().find(|e| e.get("stage") == Some(&Value::from("preflight.summary")) && e.get("decision") == Some(&Value::from("failure")))
        .expect("preflight.summary failure event");
    let chain = summary.get("summary_error_ids").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let chain_s: Vec<String> = chain.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
    assert!(chain_s.iter().any(|x| x == "E_OWNERSHIP"), "expected E_OWNERSHIP in summary_error_ids chain: {:?}", chain_s);
}
