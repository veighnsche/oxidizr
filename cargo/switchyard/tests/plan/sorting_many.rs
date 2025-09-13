//! E2E-PLAN-002 — Deterministic sorting for mixed actions (REQ-D1)
//! E2E-PLAN-005 — Many restores sorting (REQ-D1)

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput, RestoreRequest};
use switchyard::types::safepath::SafePath;

#[test]
fn plan_sorts_mixed_actions_deterministically() {
    let api = switchyard::Switchyard::new(
        JsonlSink::default(),
        JsonlSink::default(),
        Policy::default(),
    );

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    // Build 10 links and 1 restore in random-ish order (we don't rely on RNG for determinism)
    let mut inputs: Vec<(std::path::PathBuf, std::path::PathBuf)> = Vec::new();
    for i in (0..10).rev() {
        // reverse to randomize
        let src = root.join(format!("bin/new{}", i));
        let tgt = root.join(format!("usr/bin/app{}", i));
        std::fs::create_dir_all(src.parent().unwrap()).unwrap();
        std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
        std::fs::write(&src, format!("n{}", i)).unwrap();
        std::fs::write(&tgt, format!("o{}", i)).unwrap();
        inputs.push((src, tgt));
    }
    // Add one restore target
    let restore_t = root.join("usr/bin/restore_me");
    std::fs::write(&restore_t, b"r").unwrap();

    let mut link_reqs = Vec::new();
    for (s, t) in inputs {
        let sp_s = SafePath::from_rooted(root, &s).unwrap();
        let sp_t = SafePath::from_rooted(root, &t).unwrap();
        link_reqs.push(LinkRequest {
            source: sp_s,
            target: sp_t,
        });
    }
    let plan = api.plan(PlanInput {
        link: link_reqs,
        restore: vec![RestoreRequest {
            target: SafePath::from_rooted(root, &restore_t).unwrap(),
        }],
    });

    // Verify sorted by kind (EnsureSymlink first) then by target.rel lexicographically
    let mut last_kind = 0u8; // 0 for link, 1 for restore
    let mut last_t = String::new();
    for act in plan.actions.iter() {
        match act {
            switchyard::types::plan::Action::EnsureSymlink { target, .. } => {
                assert_eq!(
                    last_kind, 0u8,
                    "links should come first until restore entries"
                );
                let cur = target.rel().to_string_lossy().to_string();
                assert!(
                    cur >= last_t,
                    "targets should be sorted: {} >= {}",
                    cur,
                    last_t
                );
                last_t = cur;
            }
            switchyard::types::plan::Action::RestoreFromBackup { target } => {
                // Starting restore: bump kind and reset key ordering
                if last_kind == 0u8 {
                    last_kind = 1u8;
                    last_t.clear();
                }
                let cur = target.rel().to_string_lossy().to_string();
                assert!(
                    cur >= last_t,
                    "restore targets sorted: {} >= {}",
                    cur,
                    last_t
                );
                last_t = cur;
            }
        }
    }
}

#[test]
fn plan_many_restores_sorted() {
    let api = switchyard::Switchyard::new(
        JsonlSink::default(),
        JsonlSink::default(),
        Policy::default(),
    );
    let td = tempfile::tempdir().unwrap();
    let root = td.path();

    let mut restores = Vec::new();
    for i in (0..10).rev() {
        let t = root.join(format!("usr/bin/r{}", i));
        std::fs::create_dir_all(t.parent().unwrap()).unwrap();
        std::fs::write(&t, b"x").unwrap();
        restores.push(RestoreRequest {
            target: SafePath::from_rooted(root, &t).unwrap(),
        });
    }
    let plan = api.plan(PlanInput {
        link: vec![],
        restore: restores,
    });
    // Ensure sorted ascending
    let mut last = String::new();
    for act in plan.actions.iter() {
        match act {
            switchyard::types::plan::Action::RestoreFromBackup { target } => {
                let cur = target.rel().to_string_lossy().to_string();
                assert!(cur >= last, "restore targets sorted: {} >= {}", cur, last);
                last = cur;
            }
            _ => {}
        }
    }
}
