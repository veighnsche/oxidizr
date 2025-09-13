use log::Level;
use serde_json::Value;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;
use switchyard::{logging::AuditSink, logging::FactsEmitter, Switchyard};

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

#[derive(Default, Clone)]
struct TestAudit;
impl AuditSink for TestAudit {
    fn log(&self, _level: Level, _msg: &str) {}
}

fn plan_one(root: &std::path::Path) -> (PlanInput, std::path::PathBuf, std::path::PathBuf) {
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"new").unwrap();
    let sp_src = SafePath::from_rooted(root, &src).unwrap();
    let sp_tgt = SafePath::from_rooted(root, &tgt).unwrap();
    (
        PlanInput {
            link: vec![LinkRequest {
                source: sp_src,
                target: sp_tgt,
            }],
            restore: vec![],
        },
        src,
        tgt,
    )
}

fn run_and_get_events(require_durable: bool) -> Vec<(String, String, String, Value)> {
    let facts = TestEmitter::default();
    let audit = TestAudit::default();
    let mut policy = Policy::default();
    policy.governance.allow_unlocked_commit = true;
    policy.durability.backup_durability = require_durable;
    let api = Switchyard::new(facts.clone(), audit, policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let (input, _src, _tgt) = plan_one(root);

    let plan = api.plan(input);
    // preflight and dry-run apply
    let _ = api.preflight(&plan).unwrap();
    let _ = api.apply(&plan, ApplyMode::DryRun).unwrap();

    let evs = facts.events.lock().unwrap();
    evs.clone()
}

#[test]
fn backup_durable_flag_true_when_required() {
    let evs = run_and_get_events(true);
    assert!(evs.iter().any(|(_, event, _, f)| event == "apply.attempt"
        && f.get("backup_durable").and_then(|v| v.as_bool()) == Some(true)));
    assert!(evs.iter().any(|(_, event, _, f)| event == "apply.result"
        && f.get("backup_durable").and_then(|v| v.as_bool()) == Some(true)));
}

#[test]
fn backup_durable_flag_false_when_not_required() {
    let evs = run_and_get_events(false);
    assert!(evs.iter().any(|(_, event, _, f)| event == "apply.attempt"
        && f.get("backup_durable").and_then(|v| v.as_bool()) == Some(false)));
    assert!(evs.iter().any(|(_, event, _, f)| event == "apply.result"
        && f.get("backup_durable").and_then(|v| v.as_bool()) == Some(false)));
}
