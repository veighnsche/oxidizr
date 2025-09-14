use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use switchyard::api::Switchyard;
use switchyard::logging::{AuditSink, FactsEmitter};
use switchyard::policy::Policy;
use switchyard::types::plan::{ApplyMode, LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Default)]
struct TestEmitter {
    pub events: Arc<Mutex<Vec<Value>>>,
}

impl FactsEmitter for TestEmitter {
    fn emit(&self, _subsystem: &str, _event: &str, _decision: &str, fields: Value) {
        if let Ok(mut guard) = self.events.lock() {
            guard.push(fields);
        }
    }
}

impl AuditSink for TestEmitter {
    fn log(&self, _level: log::Level, _msg: &str) {}
}

fn compile_schema() -> JSONSchema {
    let schema_text = std::fs::read_to_string("SPEC/audit_event.v2.schema.json")
        .expect("failed to read SPEC/audit_event.v2.schema.json");
    let schema_json: Value = serde_json::from_str(&schema_text).expect("invalid JSON schema");
    JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema_json)
        .expect("failed to compile JSON schema")
}

#[test]
fn schema_v2_validates_preflight_apply_and_prune() {
    // Arrange: build API with capturing emitters
    let facts = TestEmitter::default();
    let audit = facts.clone();
    let api = Switchyard::builder(facts.clone(), audit.clone(), Policy::default()).build();

    // Plan: one ensure_symlink in a temp dir
    let td = tempfile::tempdir().expect("tempdir");
    let root = td.path();
    let src = root.join("src.bin");
    let tgt = root.join("tgt.bin");
    std::fs::write(&src, b"data").expect("write src");

    let sp_src = SafePath::from_rooted(root, &src).expect("sp src");
    let sp_tgt = SafePath::from_rooted(root, &tgt).expect("sp tgt");

    let mut input = PlanInput::default();
    input.link.push(LinkRequest { source: sp_src.clone(), target: sp_tgt.clone() });
    let plan = api.plan(input);

    // Act: preflight (always dry-run)
    let _pre = api.preflight(&plan).expect("preflight");
    // Act: apply in DryRun to avoid real FS mutations
    let _rep = api.apply(&plan, ApplyMode::DryRun).expect("apply dry-run");
    // Act: prune on target to emit prune.result; zero counts expected
    let _pr = api.prune_backups(&sp_tgt).expect("prune");

    // Validate captured events
    let compiled = compile_schema();
    let captured = facts.events.lock().expect("lock");
    assert!(captured.len() > 0, "no events captured");
    for (idx, evt) in captured.iter().enumerate() {
        if let Err(errors) = compiled.validate(evt) {
            let mut msgs = Vec::new();
            for e in errors {
                msgs.push(e.to_string());
            }
            panic!("schema validation failed for event #{idx}: {}\nEvent: {}", msgs.join("; "), serde_json::to_string_pretty(evt).unwrap());
        }
    }
}
