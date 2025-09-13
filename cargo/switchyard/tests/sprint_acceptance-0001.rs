use jsonschema::JSONSchema;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use switchyard::adapters::FsOwnershipOracle;
use switchyard::logging::{redact_event, FactsEmitter, JsonlSink};
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[derive(Default, Clone, Debug)]
struct TestEmitter {
    events: std::sync::Arc<std::sync::Mutex<Vec<(String, String, String, Value)>>>,
}

#[test]
fn preflight_rows_schema_and_ordering() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    // Build policy allowing only usr/bin so that usr/sbin violates allow_roots
    let mut policy = Policy::default();
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/sbin")).unwrap();
    std::fs::write(root.join("bin/new1"), b"n1").unwrap();
    std::fs::write(root.join("bin/new2"), b"n2").unwrap();
    std::fs::write(root.join("usr/bin/app1"), b"o1").unwrap();
    std::fs::write(root.join("usr/sbin/app2"), b"o2").unwrap();
    policy.scope.allow_roots.push(root.join("usr/bin"));
    // Allow Commit path without a LockManager so we reach policy gating
    policy.governance.allow_unlocked_commit = true;
    policy.governance.allow_unlocked_commit = true; // allow Commit without LockManager for this test

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_ownership_oracle(Box::new(FsOwnershipOracle::default()));

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

    let pf = api.preflight(&plan).unwrap();
    assert_eq!(pf.rows.len(), 2);
    // At least one row should have policy_ok=false
    assert!(
        pf.rows
            .iter()
            .any(|r| r.get("policy_ok") == Some(&Value::from(false))),
        "expected a policy_ok=false row"
    );
    // Check required keys present
    for r in &pf.rows {
        let o = r.as_object().expect("row object");
        assert!(o.get("action_id").is_some());
        assert!(o.get("path").is_some());
        assert!(o.get("current_kind").is_some());
        assert!(o.get("planned_kind").is_some());
        assert!(o.get("policy_ok").is_some());
    }
    // Verify stable ordering by (path, action_id)
    let mut sorted = pf.rows.clone();
    sorted.sort_by(|a, b| {
        let pa = a.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let pb = b.get("path").and_then(|v| v.as_str()).unwrap_or("");
        match pa.cmp(pb) {
            std::cmp::Ordering::Equal => {
                let aa = a.get("action_id").and_then(|v| v.as_str()).unwrap_or("");
                let ab = b.get("action_id").and_then(|v| v.as_str()).unwrap_or("");
                aa.cmp(ab)
            }
            other => other,
        }
    });
    assert_eq!(
        pf.rows, sorted,
        "rows should be deterministically ordered by (path, action_id)"
    );
}

#[test]
fn apply_fail_closed_on_policy_violation() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    // Ensure Commit mode reaches policy gating (bypass LockManager requirement in tests)
    policy.governance.allow_unlocked_commit = true;
    // Fail-closed default; allow only usr/bin so usr/sbin target violates policy
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/sbin")).unwrap();
    std::fs::write(root.join("bin/new"), b"new").unwrap();
    std::fs::write(root.join("usr/sbin/app"), b"old").unwrap();
    policy.scope.allow_roots.push(root.join("usr/bin"));

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_ownership_oracle(Box::new(FsOwnershipOracle::default()));

    let s = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(root, &root.join("usr/sbin/app")).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t,
        }],
        restore: vec![],
    });

    let report = api.apply(&plan, ApplyMode::Commit).unwrap();
    assert!(
        !report.errors.is_empty(),
        "apply should stop due to policy violation"
    );

    // Redacted events should include a failure apply.result with E_POLICY and exit code 10
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();
    assert!(
        redacted.iter().any(|e| {
            e.get("stage") == Some(&Value::from("apply.result"))
                && e.get("decision") == Some(&Value::from("failure"))
                && e.get("error_id") == Some(&Value::from("E_POLICY"))
                && e.get("exit_code") == Some(&Value::from(10))
        }),
        "expected E_POLICY failure with exit_code=10 in apply.result"
    );
}

#[test]
fn golden_two_action_plan_preflight_apply() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    // Setup temp tree with two actions
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/sbin")).unwrap();
    std::fs::write(root.join("bin/new1"), b"n1").unwrap();
    std::fs::write(root.join("bin/new2"), b"n2").unwrap();
    std::fs::write(root.join("usr/bin/app1"), b"o1").unwrap();
    std::fs::write(root.join("usr/sbin/app2"), b"o2").unwrap();

    // Build policy allowing only usr/bin so that usr/sbin action violates allow_roots (policy_ok=false)
    let mut policy = Policy::default();
    policy.apply.exdev = switchyard::policy::types::ExdevPolicy::DegradedFallback;
    policy.scope.allow_roots.push(root.join("usr/bin"));

    let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
        .with_ownership_oracle(Box::new(FsOwnershipOracle::default()));

    // Build plan with two link actions
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

    // Preflight + Apply(DryRun)
    let _ = api.preflight(&plan).unwrap();
    let _ = api.apply(&plan, ApplyMode::DryRun).unwrap();

    // Schema validation
    let schema_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("SPEC/audit_event.v2.schema.json");
    let schema_text =
        std::fs::read_to_string(&schema_path).expect("load SPEC/audit_event.v2.schema.json");
    let schema_json: Value = serde_json::from_str(&schema_text).expect("parse schema json");
    let compiled = JSONSchema::compile(&schema_json).expect("compile schema");
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();
    for v in &redacted {
        let errs: Option<Vec<String>> = {
            match compiled.validate(v) {
                Ok(()) => None,
                Err(it) => Some(it.map(|e| e.to_string()).collect()),
            }
        };
        if let Some(messages) = errs {
            for m in &messages {
                eprintln!(" - {}", m);
            }
            eprintln!(
                "Schema validation failed:\n{}",
                serde_json::to_string_pretty(v).unwrap()
            );
            panic!("schema validation failed");
        }
    }

    // Canon build
    let mut got: Vec<Value> = redacted
        .iter()
        .map(|f| {
            let mut v = f.clone();
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

    // Assert that at least one preflight per-action event has policy_ok=false (outside allowed roots)
    assert!(
        got.iter().any(|e| {
            e.get("stage") == Some(&Value::from("preflight"))
                && e.get("action_id").is_some()
                && e.get("policy_ok") == Some(&Value::from(false))
        }),
        "expected a preflight policy_ok=false for an out-of-allowed-roots target"
    );

    // Optionally write canon goldens
    if let Ok(outdir) = std::env::var("GOLDEN_OUT_DIR") {
        let out = std::path::Path::new(&outdir);
        std::fs::create_dir_all(out).unwrap();

        let mut canon_plan: Vec<Value> = got.iter().filter_map(|v| {
            let o = v.as_object()?; if o.get("stage") == Some(&Value::from("plan")) && o.get("action_id").is_some() { Some(serde_json::json!({"stage":"plan","action_id":o.get("action_id").cloned().unwrap()})) } else { None }
        }).collect();
        let mut canon_preflight: Vec<Value> = got.iter().filter_map(|v| {
            let o = v.as_object()?; if o.get("stage") == Some(&Value::from("preflight")) && o.get("action_id").is_some() { Some(serde_json::json!({"stage":"preflight","action_id":o.get("action_id").cloned().unwrap()})) } else { None }
        }).collect();
        let mut canon_apply_attempt: Vec<Value> = got.iter().filter_map(|v| {
            let o = v.as_object()?; if o.get("stage") == Some(&Value::from("apply.attempt")) && o.get("action_id").is_some() { Some(serde_json::json!({"stage":"apply.attempt","action_id":o.get("action_id").cloned().unwrap()})) } else { None }
        }).collect();
        let mut canon_apply_result: Vec<Value> = got.iter().filter_map(|v| {
            let o = v.as_object()?; if o.get("stage") == Some(&Value::from("apply.result")) && o.get("action_id").is_some() { Some(serde_json::json!({"stage":"apply.result","action_id":o.get("action_id").cloned().unwrap(),"decision":o.get("decision").cloned().unwrap_or(Value::from(""))})) } else { None }
        }).collect();
        let k = |v: &Value| {
            format!(
                "{}:{}",
                v.get("stage").and_then(|s| s.as_str()).unwrap_or(""),
                v.get("action_id").and_then(|s| s.as_str()).unwrap_or("")
            )
        };
        canon_plan.sort_by(|a, b| k(a).cmp(&k(b)));
        canon_preflight.sort_by(|a, b| k(a).cmp(&k(b)));
        canon_apply_attempt.sort_by(|a, b| k(a).cmp(&k(b)));
        canon_apply_result.sort_by(|a, b| k(a).cmp(&k(b)));

        let write_pretty = |path: &std::path::Path, val: &Vec<Value>| {
            let mut f = File::create(path).unwrap();
            let s = serde_json::to_string_pretty(val).unwrap();
            f.write_all(s.as_bytes()).unwrap();
        };
        write_pretty(&out.join("canon_plan.json"), &canon_plan);
        write_pretty(&out.join("canon_preflight.json"), &canon_preflight);
        write_pretty(&out.join("canon_apply_attempt.json"), &canon_apply_attempt);
        write_pretty(&out.join("canon_apply_result.json"), &canon_apply_result);
    }
}

#[test]
fn golden_determinism_dryrun_equals_commit() {
    let facts = TestEmitter::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.apply.exdev = switchyard::policy::types::ExdevPolicy::DegradedFallback;
    // Commit path now enforces preflight fail-closed unless overridden.
    // Permit untrusted (non-root-owned) sources in this deterministic test.
    policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::AllowUntrusted;
    policy.governance.allow_unlocked_commit = true; // allow Commit without LockManager to compare DryRun vs Commit

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
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: src,
            target: tgt,
        }],
        restore: vec![],
    });

    // Run DryRun
    let _ = api.preflight(&plan).unwrap();
    let _ = api.apply(&plan, ApplyMode::DryRun).unwrap();

    // Load and compile JSON Schema (once per test)
    let schema_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("SPEC/audit_event.v2.schema.json");
    let schema_text =
        std::fs::read_to_string(&schema_path).expect("load SPEC/audit_event.v2.schema.json");
    let schema_json: Value = serde_json::from_str(&schema_text).expect("parse schema json");
    let compiled = JSONSchema::compile(&schema_json).expect("compile schema");

    // Validate all redacted events (DryRun)
    for (_, _, _, f) in facts.events.lock().unwrap().iter() {
        let v = redact_event(f.clone());
        // Use inner scope to ensure the temporary from validate(&v) is dropped
        let errs: Option<Vec<String>> = {
            match compiled.validate(&v) {
                Ok(()) => None,
                Err(it) => Some(it.map(|e| e.to_string()).collect()),
            }
        };
        if let Some(messages) = errs {
            for m in &messages {
                eprintln!(" - {}", m);
            }
            eprintln!(
                "Schema validation failed (DryRun):\n{}",
                serde_json::to_string_pretty(&v).unwrap()
            );
            panic!("schema validation failed (DryRun)");
        }
    }

    // Capture and normalize DryRun events (canonical apply.result per-action only)
    let mut dry: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .filter_map(|(_, _, _, f)| {
            let mut v = redact_event(f.clone());
            if let Some(o) = v.as_object_mut() {
                if o.get("stage").and_then(|s| s.as_str()) != Some("apply.result") {
                    return None;
                }
                if !o.contains_key("action_id") {
                    return None;
                }
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
    let _plan2 = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: src2,
            target: tgt2,
        }],
        restore: vec![],
    });
    let _ = api.preflight(&plan).unwrap();
    let _ = api.apply(&plan, ApplyMode::Commit).unwrap();

    // Validate all redacted events (Commit)
    for (_, _, _, f) in facts.events.lock().unwrap().iter() {
        let v = redact_event(f.clone());
        let errs: Option<Vec<String>> = {
            match compiled.validate(&v) {
                Ok(()) => None,
                Err(it) => Some(it.map(|e| e.to_string()).collect()),
            }
        };
        if let Some(messages) = errs {
            for m in &messages {
                eprintln!(" - {}", m);
            }
            eprintln!(
                "Schema validation failed (Commit):\n{}",
                serde_json::to_string_pretty(&v).unwrap()
            );
            panic!("schema validation failed (Commit)");
        }
    }

    // Capture and normalize Commit events (canonical apply.result per-action only)
    let mut com: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .filter_map(|(_, _, _, f)| {
            let mut v = redact_event(f.clone());
            if let Some(o) = v.as_object_mut() {
                if o.get("stage").and_then(|s| s.as_str()) != Some("apply.result") {
                    return None;
                }
                if !o.contains_key("action_id") {
                    return None;
                }
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
                o.get("decision")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
            ))
        })
        .collect();
    let mut com_pairs: Vec<(String, String)> = com
        .iter()
        .filter_map(|v| {
            let o = v.as_object()?;
            Some((
                o.get("action_id")?.as_str()?.to_string(),
                o.get("decision")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
            ))
        })
        .collect();
    dry_pairs.sort_by(|a, b| a.0.cmp(&b.0));
    com_pairs.sort_by(|a, b| a.0.cmp(&b.0));
    println!("dry_pairs={:?}", dry_pairs);
    println!("com_pairs={:?}", com_pairs);
    assert_eq!(
        dry_pairs, com_pairs,
        "DryRun and Commit facts should match after redaction"
    );
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
    policy.apply.exdev = switchyard::policy::types::ExdevPolicy::DegradedFallback;

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
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: src,
            target: tgt,
        }],
        restore: vec![],
    });

    // Preflight + Apply(DryRun)
    let _ = api.preflight(&plan).unwrap();
    let _ = api.apply(&plan, ApplyMode::DryRun).unwrap();

    // Load and compile JSON Schema (v2)
    let schema_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("SPEC/audit_event.v2.schema.json");
    let schema_text =
        std::fs::read_to_string(&schema_path).expect("load SPEC/audit_event.v2.schema.json");
    let schema_json: Value = serde_json::from_str(&schema_text).expect("parse schema json");
    let compiled = JSONSchema::compile(&schema_json).expect("compile schema");

    // Validate all redacted events against schema, then canonicalize (remove path) for minimal checks.
    let redacted: Vec<Value> = facts
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, _, f)| redact_event(f.clone()))
        .collect();
    for v in &redacted {
        let errs: Option<Vec<String>> = {
            match compiled.validate(v) {
                Ok(()) => None,
                Err(it) => Some(it.map(|e| e.to_string()).collect()),
            }
        };
        if let Some(messages) = errs {
            for m in &messages {
                eprintln!(" - {}", m);
            }
            eprintln!(
                "Schema validation failed:\n{}",
                serde_json::to_string_pretty(v).unwrap()
            );
            panic!("schema validation failed");
        }
    }

    // Canonicalize for minimal presence checks and golden writing
    let mut got: Vec<Value> = redacted
        .iter()
        .map(|f| {
            let mut v = f.clone();
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
    assert!(got
        .iter()
        .any(|e| e.get("stage") == Some(&Value::from("plan"))));
    assert!(got
        .iter()
        .any(|e| e.get("stage") == Some(&Value::from("preflight"))));
    assert!(got
        .iter()
        .any(|e| e.get("stage") == Some(&Value::from("apply.attempt"))));
    assert!(got
        .iter()
        .any(|e| e.get("stage") == Some(&Value::from("apply.result"))));
    for e in &got {
        assert_eq!(e.get("schema_version"), Some(&Value::from(2)));
        assert!(e.get("plan_id").is_some());
        assert!(e.get("decision").is_some());
    }

    // Additional sprint-0001 coverage: preflight per-action emits planned_kind/current_kind, policy_ok and preservation_supported
    let mut saw_pf = false;
    for e in &got {
        if e.get("stage") == Some(&Value::from("preflight")) && e.get("action_id").is_some() {
            saw_pf = true;
            assert_eq!(e.get("planned_kind"), Some(&Value::from("symlink")));
            assert!(e.get("current_kind").is_some());
            assert!(e.get("preservation_supported").is_some());
            // policy_ok/provenance optional in schema, assert at least one present for coverage
            assert!(e.get("policy_ok").is_some() || e.get("provenance").is_some());
        }
    }
    assert!(saw_pf, "expected preflight events");

    // Optionally write canon golden files when GOLDEN_OUT_DIR is set.
    if let Ok(outdir) = std::env::var("GOLDEN_OUT_DIR") {
        let out = std::path::Path::new(&outdir);
        std::fs::create_dir_all(out).unwrap();

        // Build per-stage canon arrays with minimal stable fields.
        // plan: per-action plan facts -> keep stage + action_id
        let mut canon_plan: Vec<Value> = got
            .iter()
            .filter_map(|v| {
                let o = v.as_object()?;
                if o.get("stage") == Some(&Value::from("plan")) && o.get("action_id").is_some() {
                    let aid = o.get("action_id").cloned().unwrap_or(Value::from(""));
                    Some(serde_json::json!({"stage": "plan", "action_id": aid}))
                } else {
                    None
                }
            })
            .collect();
        // preflight: per-action preflight facts -> keep stage + action_id
        let mut canon_preflight: Vec<Value> = got
            .iter()
            .filter_map(|v| {
                let o = v.as_object()?;
                if o.get("stage") == Some(&Value::from("preflight")) && o.get("action_id").is_some()
                {
                    let aid = o.get("action_id").cloned().unwrap_or(Value::from(""));
                    Some(serde_json::json!({"stage": "preflight", "action_id": aid}))
                } else {
                    None
                }
            })
            .collect();
        // apply.attempt: per-action attempts (those with action_id) -> keep stage + action_id
        let mut canon_apply_attempt: Vec<Value> = got
            .iter()
            .filter_map(|v| {
                let o = v.as_object()?;
                if o.get("stage") == Some(&Value::from("apply.attempt"))
                    && o.get("action_id").is_some()
                {
                    let aid = o.get("action_id").cloned().unwrap_or(Value::from(""));
                    Some(serde_json::json!({"stage": "apply.attempt", "action_id": aid}))
                } else {
                    None
                }
            })
            .collect();
        // apply.result: per-action results (those with action_id) -> keep stage + action_id + decision
        let mut canon_apply_result: Vec<Value> = got
            .iter()
            .filter_map(|v| {
                let o = v.as_object()?;
                if o.get("stage") == Some(&Value::from("apply.result")) && o.get("action_id").is_some() {
                    let decision = o.get("decision").cloned().unwrap_or(Value::from(""));
                    let aid = o.get("action_id").cloned().unwrap_or(Value::from(""));
                    Some(serde_json::json!({"stage": "apply.result", "action_id": aid, "decision": decision}))
                } else { None }
            })
            .collect();

        // Sort by (stage, action_id) for determinism.
        let key2 = |v: &Value| {
            let s = v.get("stage").and_then(|x| x.as_str()).unwrap_or("");
            let a = v.get("action_id").and_then(|x| x.as_str()).unwrap_or("");
            format!("{}:{}", s, a)
        };
        canon_plan.sort_by_key(|a| key2(a));
        canon_preflight.sort_by_key(|a| key2(a));
        canon_apply_attempt.sort_by_key(|a| key2(a));
        canon_apply_result.sort_by_key(|a| key2(a));

        // Write files
        let write_pretty = |path: &std::path::Path, val: &Vec<Value>| {
            let mut f = File::create(path).unwrap();
            let s = serde_json::to_string_pretty(val).unwrap();
            f.write_all(s.as_bytes()).unwrap();
        };
        write_pretty(&out.join("canon_plan.json"), &canon_plan);
        write_pretty(&out.join("canon_preflight.json"), &canon_preflight);
        write_pretty(&out.join("canon_apply_attempt.json"), &canon_apply_attempt);
        write_pretty(&out.join("canon_apply_result.json"), &canon_apply_result);
    }
}
