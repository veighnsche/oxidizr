use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;
use switchyard::{
    adapters::{DefaultSmokeRunner, FileLockManager},
    Switchyard,
};

fn build_api(
    mut policy: Policy,
) -> Switchyard<switchyard::logging::JsonlSink, switchyard::logging::JsonlSink> {
    // Keep test hermetic: allow unlocked commit and ignore smoke by using DryRun preflight only.
    policy.allow_unlocked_commit = true;
    let facts = switchyard::logging::JsonlSink::default();
    let audit = switchyard::logging::JsonlSink::default();
    let api = Switchyard::new(facts, audit, policy)
        .with_smoke_runner(Box::new(DefaultSmokeRunner::default()))
        .with_lock_manager(Box::new(FileLockManager::new(
            std::env::temp_dir().join("lock"),
        )));
    api
}

#[test]
fn preflight_stops_on_suid_sgid_when_disallowed() {
    let mut policy = Policy::default();
    // Avoid unrelated STOP from untrusted source
    policy.force_untrusted_source = true;
    let api = build_api(policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"new").unwrap();
    std::fs::write(&tgt, b"old").unwrap();
    // Set SUID bit on target
    let mut perm = std::fs::metadata(&tgt).unwrap().permissions();
    use std::os::unix::fs::PermissionsExt;
    perm.set_mode(0o4755);
    std::fs::set_permissions(&tgt, perm).unwrap();

    let sp_src = SafePath::from_rooted(root, &src).unwrap();
    let sp_tgt = SafePath::from_rooted(root, &tgt).unwrap();
    let input = PlanInput {
        link: vec![LinkRequest {
            source: sp_src,
            target: sp_tgt,
        }],
        restore: vec![],
    };
    let plan = api.plan(input);
    let report = api.preflight(&plan).unwrap();
    assert!(
        report.stops.iter().any(|s| s.contains("suid/sgid risk")),
        "expected STOP on suid/sgid risk: {:?}",
        report.stops
    );
}

#[test]
fn preflight_warns_on_suid_sgid_when_allowed() {
    let mut policy = Policy::default();
    policy.force_untrusted_source = true;
    policy.allow_suid_sgid_mutation = true;
    let api = build_api(policy);

    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
    std::fs::write(&src, b"new").unwrap();
    std::fs::write(&tgt, b"old").unwrap();
    // Set SGID bit on target
    let mut perm = std::fs::metadata(&tgt).unwrap().permissions();
    use std::os::unix::fs::PermissionsExt;
    perm.set_mode(0o2755);
    std::fs::set_permissions(&tgt, perm).unwrap();

    let sp_src = SafePath::from_rooted(root, &src).unwrap();
    let sp_tgt = SafePath::from_rooted(root, &tgt).unwrap();
    let input = PlanInput {
        link: vec![LinkRequest {
            source: sp_src,
            target: sp_tgt,
        }],
        restore: vec![],
    };
    let plan = api.plan(input);
    let report = api.preflight(&plan).unwrap();
    assert!(
        report.stops.iter().all(|s| !s.contains("suid/sgid risk")),
        "should not STOP on suid/sgid when allowed: {:?}",
        report.stops
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.contains("suid/sgid risk allowed by policy")),
        "expected WARN on suid/sgid risk: {:?}",
        report.warnings
    );
}
