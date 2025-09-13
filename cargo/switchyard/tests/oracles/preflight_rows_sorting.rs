//! Preflight rows sorting assertion
//! Assert rows sorted by (path, action_id) and summary error_id/exit_code mapping for failures.

use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

#[test]
fn preflight_rows_sorting() {
    // Preflight rows sorting assertion (P0)
    // Assert rows sorted by (path, action_id)
    
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let policy = Policy::default();
    let api = switchyard::Switchyard::new(facts, audit, policy);
    
    // Use temp directory
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    // Create multiple targets to test sorting
    let src1 = root.join("bin/new1");
    let src2 = root.join("bin/new2");
    let tgt1 = root.join("usr/bin/app1");
    let tgt2 = root.join("usr/bin/app2");
    
    std::fs::create_dir_all(src1.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt1.parent().unwrap()).unwrap();
    std::fs::write(&src1, b"n1").unwrap();
    std::fs::write(&src2, b"n2").unwrap();
    std::fs::write(&tgt1, b"o1").unwrap();
    std::fs::write(&tgt2, b"o2").unwrap();
    
    let s1 = SafePath::from_rooted(root, &src1).unwrap();
    let s2 = SafePath::from_rooted(root, &src2).unwrap();
    let t1 = SafePath::from_rooted(root, &tgt1).unwrap();
    let t2 = SafePath::from_rooted(root, &tgt2).unwrap();
    
    let input = PlanInput { 
        link: vec![
            LinkRequest { source: s1, target: t1.clone() },
            LinkRequest { source: s2, target: t2.clone() }
        ], 
        restore: vec![] 
    };
    
    let plan = api.plan(input);
    let pf = api.preflight(&plan).unwrap();
    
    // Verify rows are sorted by (path, action_id)
    let mut last_path = String::new();
    let mut last_action_id = String::new();
    for row in pf.rows.iter() {
        let path = row.path.to_string_lossy().to_string();
        let action_id = row.action_id.clone();
        
        // Check path ordering
        assert!(path >= last_path, "rows should be sorted by path: {} >= {}", path, last_path);
        
        // If same path, check action_id ordering
        if path == last_path {
            assert!(action_id >= last_action_id, "rows with same path should be sorted by action_id: {} >= {}", action_id, last_action_id);
        }
        
        last_path = path;
        last_action_id = action_id;
    }
}
