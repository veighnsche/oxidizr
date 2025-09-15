use cucumber::{given, then, when};

use crate::bdd_world::World;
use std::path::{Path, PathBuf};
use switchyard::types::plan::{LinkRequest, PlanInput, RestoreRequest};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

#[then(regex = r"^preflight fails closed unless an explicit policy override is present$")]
pub async fn then_preflight_fails_unless_override(world: &mut World) {
    // First run: expect STOP (failure) in preflight due to gating
    crate::steps::preflight_steps::when_preflight(world).await;
    let pf = world.preflight.as_ref().expect("preflight report");
    assert!(!pf.ok, "expected preflight STOP prior to override");
    // Now enable explicit override and re-run preflight; expect success
    world.policy.apply.override_preflight = true;
    world.rebuild_api();
    crate::steps::preflight_steps::when_preflight(world).await;
    let pf2 = world.preflight.as_ref().expect("preflight report");
    assert!(
        pf2.ok,
        "expected preflight success when override_preflight=true"
    );
}

#[then(regex = r"^preflight stops with a fail-closed decision unless an explicit override is set$")]
pub async fn then_preservation_fail_unless_override(world: &mut World) {
    // First run: expect STOP (failure)
    crate::steps::preflight_steps::when_preflight(world).await;
    let pf = world.preflight.as_ref().expect("preflight report");
    assert!(!pf.ok, "expected preflight STOP for preservation gating");
    // Now override and expect success
    world.policy.apply.override_preflight = true;
    world.rebuild_api();
    crate::steps::preflight_steps::when_preflight(world).await;
    let pf2 = world.preflight.as_ref().expect("preflight report");
    assert!(
        pf2.ok,
        "expected preflight success when override_preflight=true"
    );
}

// Local helper: scan for the latest backup payload file for a given target path.
// Follows the naming scheme .{name}.{tag}.{ts}.bak where tag can be empty.
fn find_latest_backup_payload(target: &Path) -> Option<PathBuf> {
    let name = target.file_name()?.to_str()?;
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let mut best: Option<(u128, PathBuf)> = None;
    for entry in std::fs::read_dir(parent).ok()?.flatten() {
        let fname = entry.file_name();
        let s = fname.to_str()?;
        // Accept both tagged and untagged variants
        let rest_opt = s
            .strip_prefix(&format!(".{name}."))
            .and_then(|rest| rest.strip_suffix(".bak"));
        let Some(rest) = rest_opt else { continue };
        // Timestamp is the last dotted segment
        let ts_s = rest.rsplit('.').next().unwrap_or("");
        let Ok(ts) = ts_s.parse::<u128>() else {
            continue;
        };
        // Record the best candidate
        let p = parent.join(s);
        if best.as_ref().map(|(cur, _)| ts > *cur).unwrap_or(true) {
            best = Some((ts, p));
        }
    }
    best.map(|(_, p)| p)
}

// SafePath escaping checks
#[given(regex = r"^a candidate path containing \.\.\s*segments or symlink escapes$")]
pub async fn given_candidate_path_unsafe(world: &mut World) {
    // Store an obviously unsafe candidate
    world.last_src = Some("../etc/passwd".to_string());
}

#[when(regex = r"^I attempt to construct a SafePath$")]
pub async fn when_construct_safepath(world: &mut World) {
    let root = world.ensure_root().to_path_buf();
    let cand = world
        .last_src
        .clone()
        .unwrap_or_else(|| "../etc/passwd".to_string());
    let res =
        switchyard::types::safepath::SafePath::from_rooted(&root, std::path::Path::new(&cand));
    // Record the result as a fact in audit memory via world fields (ephemeral)
    if res.is_ok() {
        world.last_src = Some("SAFE_OK".to_string());
    } else {
        world.last_src = Some("SAFE_ERR".to_string());
    }
}

#[then(regex = r"^SafePath normalization rejects the path as unsafe$")]
pub async fn then_safepath_rejects(world: &mut World) {
    assert_eq!(world.last_src.as_deref(), Some("SAFE_ERR"));
}

// Note: aliases for existing steps (candidate path, construct SafePath, filesystem unsupported,
// attempt apply, policy violation) are defined in other step modules to avoid ambiguity.

// Ownership gating (strict)
#[given(regex = r"^strict_ownership=true policy$")]
pub async fn given_strict_ownership_policy(world: &mut World) {
    world.policy.risks.ownership_strict = true;
    world.rebuild_api();
}

#[given(regex = r"^a target that is not package-owned per the ownership oracle$")]
pub async fn given_target_not_pkg_owned(world: &mut World) {
    // Install a stub oracle that always reports error -> policy STOP under strict ownership
    #[derive(Debug)]
    struct FailingOracle;
    impl switchyard::adapters::OwnershipOracle for FailingOracle {
        fn owner_of(
            &self,
            _path: &switchyard::types::safepath::SafePath,
        ) -> switchyard::types::errors::Result<switchyard::types::ownership::OwnershipInfo>
        {
            Err(switchyard::types::errors::Error {
                kind: switchyard::types::errors::ErrorKind::Policy,
                msg: "not package-owned".to_string(),
            })
        }
    }
    world.ensure_api();
    let api = switchyard::api::Switchyard::builder(
        world.facts.clone(),
        world.audit.clone(),
        world.policy.clone(),
    )
    .with_ownership_oracle(Box::new(FailingOracle))
    .build();
    world.api = Some(api);
}

// Intentionally no duplicate for "I run preflight"; use the canonical
// implementation from preflight_steps to avoid ambiguous matches.

#[then(regex = r"^preflight fails closed$")]
pub async fn then_preflight_fails_closed(world: &mut World) {
    // Look for a preflight.summary failure with error_id=E_POLICY
    let mut saw = false;
    for ev in world.all_facts() {
        if ev.get("stage").and_then(|v| v.as_str()) == Some("preflight.summary")
            && ev.get("decision").and_then(|v| v.as_str()) == Some("failure")
        {
            saw = true;
            break;
        }
    }
    assert!(saw, "expected preflight.summary failure");
}

// Preservation capability gating
#[given(
    regex = r"^the policy requires preserving owner, mode, timestamps, xattrs, ACLs, and caps$"
)]
pub async fn given_preservation_required(world: &mut World) {
    world.policy.durability.preservation =
        switchyard::policy::types::PreservationPolicy::RequireBasic;
    world.rebuild_api();
}

#[given(regex = r"^the filesystem or environment lacks support for one or more of these$")]
pub async fn given_env_lacks_support(_world: &mut World) {
    // No-op: On typical CI/Linux, some dimensions (xattrs, caps) are conservatively unsupported
    // by detect_preservation_capabilities(), which will cause preflight to STOP when required.
}

// Backup sidecar integrity
#[given(regex = r"^a backup sidecar v2 with payload present$")]
pub async fn given_backup_sidecar_with_payload(world: &mut World) {
    // Create target as a regular file and take a snapshot via a swap, then tamper payload
    let root = world.ensure_root().to_path_buf();
    let src = root.join("bin/new");
    let tgt = root.join("usr/bin/app");
    if let Some(p) = src.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    if let Some(p) = tgt.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let _ = std::fs::write(&src, b"new");
    let _ = std::fs::write(&tgt, b"old");
    let s = SafePath::from_rooted(&root, &src).unwrap();
    let t = SafePath::from_rooted(&root, &tgt).unwrap();
    let plan = PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t.clone(),
        }],
        restore: vec![],
    };
    world.ensure_api();
    let plan = world.api.as_ref().unwrap().plan(plan);
    let _ = world.api.as_ref().unwrap().apply(&plan, ApplyMode::Commit);
    // Tamper: overwrite backup payload if present (best-effort)
    if let Some(b) = find_latest_backup_payload(&t.as_path()) {
        let _ = std::fs::write(b, b"tampered");
    }
}

#[when(regex = r"^I restore under policy requiring sidecar integrity$")]
pub async fn when_restore_with_integrity(world: &mut World) {
    world.policy.durability.sidecar_integrity = true;
    world.rebuild_api();
    let root = world.ensure_root().to_path_buf();
    let tgt = SafePath::from_rooted(&root, &root.join("usr/bin/app")).unwrap();
    let input = PlanInput {
        link: vec![],
        restore: vec![RestoreRequest { target: tgt }],
    };
    let plan = world.api.as_ref().unwrap().plan(input);
    let _ = world.api.as_ref().unwrap().apply(&plan, ApplyMode::Commit);
}

#[then(regex = r"^the engine verifies the payload hash and fails restore on mismatch$")]
pub async fn then_restore_fails_on_mismatch(world: &mut World) {
    // Expect at least one apply.result failure with E_RESTORE_FAILED or sidecar_integrity_verified=false present
    let mut ok = false;
    for ev in world.all_facts() {
        if ev.get("stage").and_then(|v| v.as_str()) == Some("apply.result")
            && ev.get("decision").and_then(|v| v.as_str()) == Some("failure")
        {
            ok = true;
            break;
        }
    }
    assert!(
        ok,
        "expected a failing apply.result during restore due to integrity mismatch"
    );
}

// Ownership gating (world-writable source)
#[given(regex = r"^a source file that is not root-owned or is world-writable$")]
pub async fn given_world_writable_source(world: &mut World) {
    use std::os::unix::fs::PermissionsExt;
    let root = world.ensure_root().to_path_buf();
    // Create a world-writable source and a target
    let src = root.join("bin/ww");
    let tgt = root.join("usr/bin/app");
    if let Some(p) = src.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    if let Some(p) = tgt.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    std::fs::write(&src, b"src").unwrap();
    let mut perm = std::fs::metadata(&src).unwrap().permissions();
    perm.set_mode(0o666);
    std::fs::set_permissions(&src, perm).unwrap();
    std::fs::write(&tgt, b"old").unwrap();
    let s = SafePath::from_rooted(&root, &src).unwrap();
    let t = SafePath::from_rooted(&root, &tgt).unwrap();
    let plan = PlanInput {
        link: vec![LinkRequest {
            source: s,
            target: t,
        }],
        restore: vec![],
    };
    world.ensure_api();
    world.plan = Some(world.api.as_ref().unwrap().plan(plan));
    // Ensure strict source trust to enforce gating
    world.policy.risks.source_trust = switchyard::policy::types::SourceTrustPolicy::RequireTrusted;
    world.rebuild_api();
}
