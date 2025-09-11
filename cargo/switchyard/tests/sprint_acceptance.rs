use switchyard; // crate name per Cargo.toml
use serde_json::Value;
use switchyard::logging::{redact_event, FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;
use switchyard::adapters::FsOwnershipOracle;

#[derive(Default, Clone)]
struct TestEmitter {
    events: std::sync::Arc<std::sync::Mutex<Vec<(String, String, String, Value)>>>,
}

#[test]
fn golden_determinism_dryrun_equals_commit() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.allow_degraded_fs = true;

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_ownership_oracle(Box::new(FsOwnershipOracle::default()));

    // Setup temp tree
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"new").unwrap();
    std::fs::write(root.join("usr/bin/ls"), b"old").unwrap();

    // Build plan
    let src = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let tgt = SafePath::from_rooted(root, &root.join("usr/bin/ls")).unwrap();
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: src, target: tgt }], restore: vec![] });

    // Run DryRun
    let _ = api.preflight(&plan).unwrap();
    let _ = api.apply(&plan, ApplyMode::DryRun).unwrap();

    // Capture and normalize DryRun events (canonical apply.result per-action only)
    let mut dry: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .filter_map(|(_, _, _, f)| {
            let mut v = redact_event(f.clone());
            if let Some(o) = v.as_object_mut() {
                if o.get("stage").and_then(|s| s.as_str()) != Some("apply.result") { return None; }
                if !o.contains_key("action_id") { return None; }
                // Canonicalize: keep only stage, action_id, decision
                let stage = o.get("stage").cloned().unwrap();
                let aid = o.get("action_id").cloned().unwrap();
                let decision = o.get("decision").cloned().unwrap_or(Value::from(""));
                *o = serde_json::json!({
                    "stage": stage,
                    "action_id": aid,
                    "decision": decision,
                })
                .as_object()
                .unwrap()
                .clone();
            }
            Some(v)
        })
        .collect();
    dry.sort_by(|a, b| {
        let ka = format!(
            "{}:{}",
            a.get("stage").and_then(|v| v.as_str()).unwrap_or(""),
            a.get("action_id").and_then(|v| v.as_str()).unwrap_or("")
        );
        let kb = format!(
            "{}:{}",
            b.get("stage").and_then(|v| v.as_str()).unwrap_or(""),
            b.get("action_id").and_then(|v| v.as_str()).unwrap_or("")
        );
        ka.cmp(&kb)
    });

    // Clear captured events, then re-emit plan and run Commit (stage parity)
    facts.events.lock().unwrap().clear();
    // Rebuild SafePaths and re-emit plan facts
    let src2 = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let tgt2 = SafePath::from_rooted(root, &root.join("usr/bin/ls")).unwrap();
    let _plan2 = api.plan(PlanInput { link: vec![LinkRequest { source: src2, target: tgt2 }], restore: vec![] });
    let _ = api.preflight(&plan).unwrap();
    let _ = api.apply(&plan, ApplyMode::Commit).unwrap();

    // Capture and normalize Commit events (canonical apply.result per-action only)
    let mut com: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .filter_map(|(_, _, _, f)| {
            let mut v = redact_event(f.clone());
            if let Some(o) = v.as_object_mut() {
                if o.get("stage").and_then(|s| s.as_str()) != Some("apply.result") { return None; }
                if !o.contains_key("action_id") { return None; }
                // Canonicalize: keep only stage, action_id, decision
                let stage = o.get("stage").cloned().unwrap();
                let aid = o.get("action_id").cloned().unwrap();
                let decision = o.get("decision").cloned().unwrap_or(Value::from(""));
                *o = serde_json::json!({
                    "stage": stage,
                    "action_id": aid,
                    "decision": decision,
                })
                .as_object()
                .unwrap()
                .clone();
            }
            Some(v)
        })
        .collect();
    com.sort_by(|a, b| {
        let ka = format!(
            "{}:{}",
            a.get("stage").and_then(|v| v.as_str()).unwrap_or(""),
            a.get("action_id").and_then(|v| v.as_str()).unwrap_or("")
        );
        let kb = format!(
            "{}:{}",
            b.get("stage").and_then(|v| v.as_str()).unwrap_or(""),
            b.get("action_id").and_then(|v| v.as_str()).unwrap_or("")
        );
        ka.cmp(&kb)
    });

    // Debug: print per-index diffs when mismatch
    if dry.len() != com.len() {
        eprintln!("event count differs: dry={} com={}", dry.len(), com.len());
    }
    let n = std::cmp::min(dry.len(), com.len());
    for i in 0..n {
        if dry[i] != com[i] {
            eprintln!(
                "diff at idx {}:\nDRY=\n{}\nCOM=\n{}\n",
                i,
                serde_json::to_string_pretty(&dry[i]).unwrap(),
                serde_json::to_string_pretty(&com[i]).unwrap()
            );
        }
    }
    // Compare canonical tuples (action_id, decision)
    let mut dry_pairs: Vec<(String, String)> = dry
        .iter()
        .filter_map(|v| {
            let o = v.as_object()?;
            Some((
                o.get("action_id")?.as_str()?.to_string(),
                o.get("decision").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            ))
        })
        .collect();
    let mut com_pairs: Vec<(String, String)> = com
        .iter()
        .filter_map(|v| {
            let o = v.as_object()?;
            Some((
                o.get("action_id")?.as_str()?.to_string(),
                o.get("decision").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            ))
        })
        .collect();
    dry_pairs.sort_by(|a, b| a.0.cmp(&b.0));
    com_pairs.sort_by(|a, b| a.0.cmp(&b.0));
    println!("dry_pairs={:?}", dry_pairs);
    println!("com_pairs={:?}", com_pairs);
    assert_eq!(dry_pairs, com_pairs, "DryRun and Commit facts should match after redaction");
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
fn golden_minimal_plan_preflight_apply() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.allow_degraded_fs = true;

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_ownership_oracle(Box::new(FsOwnershipOracle::default()));

    // Setup temp tree
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"new").unwrap();
    std::fs::write(root.join("usr/bin/ls"), b"old").unwrap();

    // Build plan
    let src = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let tgt = SafePath::from_rooted(root, &root.join("usr/bin/ls")).unwrap();
    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: src, target: tgt }], restore: vec![] });

    // Preflight + Apply(DryRun)
    let _ = api.preflight(&plan).unwrap();
    let _ = api.apply(&plan, ApplyMode::DryRun).unwrap();

    // Normalize events and compare to a minimal golden structure
    let mut got: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| {
            let mut v = redact_event(f.clone());
            if let Some(o) = v.as_object_mut() {
                o.remove("path");
            }
            v
        })
        .collect();
    got.sort_by(|a, b| {
        let ka = format!(
            "{}:{}",
            a.get("stage").and_then(|v| v.as_str()).unwrap_or(""),
            a.get("action_id").and_then(|v| v.as_str()).unwrap_or("")
        );
        let kb = format!(
            "{}:{}",
            b.get("stage").and_then(|v| v.as_str()).unwrap_or(""),
            b.get("action_id").and_then(|v| v.as_str()).unwrap_or("")
        );
        ka.cmp(&kb)
    });

    // The golden just asserts presence of key stages with schema_version and plan_id
    assert!(got.iter().any(|e| e.get("stage") == Some(&Value::from("plan"))));
    assert!(got.iter().any(|e| e.get("stage") == Some(&Value::from("preflight"))));
    assert!(got.iter().any(|e| e.get("stage") == Some(&Value::from("apply.attempt"))));
    assert!(got.iter().any(|e| e.get("stage") == Some(&Value::from("apply.result"))));
    for e in &got {
        assert_eq!(e.get("schema_version"), Some(&Value::from(1)));
        assert!(e.get("plan_id").is_some());
        assert!(e.get("decision").is_some());
    }
}
