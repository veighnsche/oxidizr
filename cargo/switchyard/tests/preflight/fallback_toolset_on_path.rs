//! REQ-RC3 — Fallback toolset available on PATH enables preflight OK when required

use serial_test::serial;
use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{LinkRequest, PlanInput};
use switchyard::types::safepath::SafePath;

#[test]
#[serial]
fn fallback_toolset_on_path() {
    // Guard PATH to avoid leakage
    struct EnvGuard(Option<std::ffi::OsString>);
    impl EnvGuard {
        fn new() -> Self {
            Self(std::env::var_os("PATH"))
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(v) = self.0.take() {
                std::env::set_var("PATH", v)
            } else {
                std::env::remove_var("PATH")
            }
        }
    }

    let _g = EnvGuard::new();
    // Create a temp dir with a fake executable named "busybox" to satisfy rescue
    let td = tempfile::tempdir().unwrap();
    let tool = td.path().join("busybox");
    std::fs::write(&tool, b"#!/bin/sh\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&tool).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&tool, perms).unwrap();
    }
    std::env::set_var("PATH", td.path());

    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let mut policy = Policy::default();
    policy.rescue.require = true;
    policy.rescue.exec_check = true;

    let api = switchyard::Switchyard::new(facts, audit, policy);

    // Minimal plan in temp root
    let rootd = tempfile::tempdir().unwrap();
    let root = rootd.path();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("usr/bin")).unwrap();
    std::fs::write(root.join("bin/new"), b"n").unwrap();
    std::fs::write(root.join("usr/bin/app"), b"o").unwrap();

    let s = SafePath::from_rooted(root, &root.join("bin/new")).unwrap();
    let t = SafePath::from_rooted(root, &root.join("usr/bin/app")).unwrap();
    let plan = api.plan(PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t,
        }],
        restore: vec![],
    });

    let pf = api.preflight(&plan).unwrap();
    assert!(
        pf.ok,
        "preflight should succeed when a rescue toolset is available on PATH"
    );
}
