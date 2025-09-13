use serde_json::Value;

use switchyard::logging::FactsEmitter;
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
fn facts_conform_to_json_schema() {
    // Prepare API with DryRun so timestamps are deterministic
    let facts = TestEmitter::default();
    let audit = switchyard::logging::JsonlSink::default();
    let policy = Policy::default();
    let api = switchyard::Switchyard::new(facts.clone(), audit, policy);

    // Layout under tmp root
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"new").unwrap();

    let source = SafePath::from_rooted(root, &src).unwrap();
    let target = SafePath::from_rooted(root, &tgt).unwrap();
    let input = PlanInput {
        link: vec![LinkRequest { source, target }],
        restore: vec![],
    };

    let plan = api.plan(input);
    let _ = api.preflight(&plan).unwrap();
    let _ = api.apply(&plan, ApplyMode::DryRun).unwrap();

    // Load schema (v2)
    let schema_str = include_str!("../../SPEC/audit_event.v2.schema.json");
    let schema_json: serde_json::Value = serde_json::from_str(schema_str).unwrap();
    let compiled = jsonschema::JSONSchema::compile(&schema_json).expect("valid schema");

    // Validate all captured events
    let events = facts.events.lock().unwrap();
    assert!(!events.is_empty(), "no events captured");
    for (_subsystem, _event, _decision, fields) in events.iter() {
        if let Err(errors) = compiled.validate(fields) {
            eprintln!(
                "Invalid event: {}",
                serde_json::to_string_pretty(fields).unwrap()
            );
            for err in errors {
                eprintln!("  -> {}", err);
            }
            panic!("event failed schema validation");
        }
    }
}
