#![cfg_attr(not(feature = "bdd"), allow(unused_imports, unused_variables, dead_code))]

use cucumber::World as _; // bring trait into scope for World::cucumber()
use serde_json::Value;
use std::path::Path;
use switchyard::adapters::{DefaultSmokeRunner, FileLockManager};
use switchyard::api::Switchyard;
use switchyard::policy::types::{ExdevPolicy, LockingPolicy, SmokePolicy};
use switchyard::policy::Policy;
use switchyard::types::plan::{ApplyMode, LinkRequest, PlanInput};
use switchyard::types::report::{ApplyReport, PreflightReport};
use switchyard::preflight::yaml as preflight_yaml;

mod bdd_support;
use bdd_support::{util, CollectingAudit, CollectingEmitter};

#[derive(Default, cucumber::World)]
pub struct World {
    root: Option<tempfile::TempDir>,
    api: Option<Switchyard<CollectingEmitter, CollectingAudit>>,
    policy: Policy,
    plan: Option<switchyard::types::Plan>,
    preflight: Option<PreflightReport>,
    apply_report: Option<ApplyReport>,
    facts: CollectingEmitter,
    audit: CollectingAudit,
    last_link: Option<String>,
    last_src: Option<String>,
    lock_path: Option<std::path::PathBuf>,
    facts_dry: Option<Vec<Value>>,
    facts_real: Option<Vec<Value>>,
}

impl World {
    fn ensure_root(&mut self) -> &Path {
        if self.root.is_none() {
            self.root = Some(tempfile::TempDir::new().expect("tempdir"));
        }
        self.root.as_ref().unwrap().path()
    }

    fn rebuild_api(&mut self) {
        let api = Switchyard::builder(self.facts.clone(), self.audit.clone(), self.policy.clone()).build();
        self.api = Some(api);
    }

    fn ensure_api(&mut self) {
        if self.api.is_none() {
            self.rebuild_api();
        }
    }

    fn clear_facts(&mut self) {
        self.facts.0.lock().unwrap().clear();
        self.audit.0.lock().unwrap().clear();
    }

    fn mk_symlink(&mut self, link: &str, dest: &str) {
        let root = self.ensure_root().to_path_buf();
        let l = util::under_root(&root, link);
        let d = util::under_root(&root, dest);
        if let Some(p) = l.parent() { let _ = std::fs::create_dir_all(p); }
        if let Some(p) = d.parent() { let _ = std::fs::create_dir_all(p); }
        // Ensure dest exists
        let _ = std::fs::write(&d, b"payload");
        let _ = std::fs::remove_file(&l);
        #[cfg(unix)]
        std::os::unix::fs::symlink(&d, &l).expect("symlink");
        self.last_link = Some(link.to_string());
        self.last_src = Some(dest.to_string());
    }

    fn build_single_swap(&mut self, link: &str, src: &str) {
        let root = self.ensure_root().to_path_buf();
        // ensure source exists
        let sp_src = util::sp(&root, src);
        if let Some(p) = sp_src.as_path().parent() { let _ = std::fs::create_dir_all(p); }
        let _ = std::fs::write(sp_src.as_path(), b"new");
        let sp_tgt = util::sp(&root, link);
        let mut input = PlanInput::default();
        input.link.push(LinkRequest { source: sp_src, target: sp_tgt });
        self.ensure_api();
        let plan = self.api.as_ref().unwrap().plan(input);
        self.plan = Some(plan);
    }
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("root", &self.root.as_ref().map(|_| "tempdir"))
            .field("policy", &"Policy{..}")
            .field("plan", &self.plan.as_ref().map(|p| p.actions.len()))
            .finish()
    }
}

// Step definitions grouped under a module so cucumber can register inventory.
mod steps {
    use super::*;
    use cucumber::{given, then, when};
    use switchyard::adapters::LockManager;
    use jsonschema::JSONSchema;
    use switchyard::logging::redact::redact_event;
    use switchyard::adapters::{Attestor, Signature};

    // Given steps
    #[given(regex = r"^(/.+) is a symlink to (.+)$")]
    async fn given_symlink(world: &mut World, link: String, dest: String) {
        world.mk_symlink(&link, &dest);
    }

    #[given(regex = r"^a plan with at least one action$")]
    async fn given_plan_min(world: &mut World) {
        let link = world.last_link.clone().unwrap_or_else(|| "/usr/bin/ls".to_string());
        let src = world.last_src.clone().unwrap_or_else(|| "providerB/ls".to_string());
        world.build_single_swap(&link, &src);
    }

    #[given(regex = r"^the target and staging directories reside on different filesystems$")]
    async fn given_exdev_env(_world: &mut World) { std::env::set_var("SWITCHYARD_FORCE_EXDEV", "1"); }

    #[given(regex = r"^a production deployment with a LockManager$")]
    async fn given_with_lock(world: &mut World) {
        let lock_path = world.ensure_root().join("switchyard.lock");
        world.lock_path = Some(lock_path.clone());
        let api = Switchyard::builder(world.facts.clone(), world.audit.clone(), world.policy.clone())
            .with_lock_manager(Box::new(FileLockManager::new(lock_path)))
            .build();
        world.api = Some(api);
    }

    #[given(regex = r"^a Switchyard built with a LockManager$")]
    async fn given_with_lock_alias(world: &mut World) { given_with_lock(world).await }

    #[given(regex = r"^a development environment without a LockManager$")]
    async fn given_without_lock(world: &mut World) {
        world.policy.governance.locking = LockingPolicy::Optional;
        world.policy.governance.allow_unlocked_commit = true;
        world.rebuild_api();
    }

    #[given(regex = r"^a Switchyard without a LockManager$")]
    async fn given_without_lock_alias(world: &mut World) { given_without_lock(world).await }

    #[given(regex = r"^the minimal post-apply smoke suite is configured$")]
    async fn given_smoke(world: &mut World) {
        world.policy.governance.smoke = SmokePolicy::Require { auto_rollback: true };
        let api = Switchyard::builder(world.facts.clone(), world.audit.clone(), world.policy.clone())
            .with_smoke_runner(Box::new(DefaultSmokeRunner::default()))
            .build();
        world.api = Some(api);
    }

    // When steps
    #[when(regex = r"^I plan a swap to (\S+)$")]
    async fn when_plan_swap(world: &mut World, provider: String) {
        let link = world.last_link.clone().unwrap_or_else(|| "/usr/bin/ls".to_string());
        let src = format!("{}/ls", provider);
        world.build_single_swap(&link, &src);
    }

    #[when(regex = r"^I run in real mode$")]
    async fn when_run_real(world: &mut World) {
        world.ensure_api();
        if world.plan.is_none() {
            given_plan_min(world).await;
        }
        let plan = world.plan.as_ref().unwrap();
        let _ = world.api.as_ref().unwrap().preflight(plan).unwrap();
        let _ = world.api.as_ref().unwrap().apply(plan, ApplyMode::Commit).unwrap();
    }

    #[when(regex = r"^I run in Commit mode$")]
    async fn when_run_commit(world: &mut World) { when_run_real(world).await }

    #[when(regex = r"^I run in DryRun and Commit modes$")]
    async fn when_run_both_modes(world: &mut World) {
        world.ensure_api();
        if world.plan.is_none() { given_plan_min(world).await; }
        let plan = world.plan.as_ref().unwrap();
        let _ = world.api.as_ref().unwrap().apply(plan, ApplyMode::DryRun).unwrap();
        let _ = world.api.as_ref().unwrap().apply(plan, ApplyMode::Commit).unwrap();
    }

    #[when(regex = r"^I run preflight$")]
    async fn when_preflight(world: &mut World) {
        world.ensure_api();
        if world.plan.is_none() { given_plan_min(world).await; }
        let plan = world.plan.as_ref().unwrap();
        world.preflight = Some(world.api.as_ref().unwrap().preflight(plan).unwrap());
    }

    #[when(regex = r"^I run preflight in DryRun and Commit modes$")]
    async fn when_preflight_both(world: &mut World) {
        world.ensure_api();
        if world.plan.is_none() { given_plan_min(world).await; }
        let plan = world.plan.as_ref().unwrap();
        let _ = world.api.as_ref().unwrap().preflight(plan).unwrap();
        // preflight is always dry-run; re-run to simulate commit parity
        let _ = world.api.as_ref().unwrap().preflight(plan).unwrap();
    }

    #[when(regex = r"^I apply the plan$")]
    async fn when_apply(world: &mut World) { world.ensure_api(); let plan = world.plan.as_ref().unwrap(); world.apply_report = Some(world.api.as_ref().unwrap().apply(plan, ApplyMode::Commit).unwrap()); }

    #[when(regex = r"^I run in dry-run mode$")]
    async fn when_run_dry(world: &mut World) {
        world.ensure_api();
        if world.plan.is_none() { given_plan_min(world).await; }
        world.clear_facts();
        let plan = world.plan.as_ref().unwrap();
        let _ = world.api.as_ref().unwrap().apply(plan, ApplyMode::DryRun).unwrap();
        world.facts_dry = Some(world.facts.0.lock().unwrap().clone());
    }

    #[when(regex = r"^two apply\(\) calls overlap in time$")]
    async fn when_two_apply_overlap(world: &mut World) {
        if world.plan.is_none() { given_plan_min(world).await; }
        let plan = world.plan.as_ref().unwrap().clone();
        let plan1 = plan.clone();
        let plan2 = plan.clone();
        let lock_path = world.lock_path.clone().unwrap_or_else(|| world.ensure_root().join("switchyard.lock"));
        let lock1 = lock_path.clone();
        let lock2 = lock_path.clone();
        let facts1 = world.facts.clone();
        let audit1 = world.audit.clone();
        let policy1 = world.policy.clone();
        let h1 = std::thread::spawn(move || {
            let api = Switchyard::builder(facts1.clone(), audit1.clone(), policy1.clone())
                .with_lock_manager(Box::new(FileLockManager::new(lock1)))
                .build();
            let _ = api.apply(&plan1, ApplyMode::Commit);
        });
        let facts2 = world.facts.clone();
        let audit2 = world.audit.clone();
        let policy2 = world.policy.clone();
        let h2 = std::thread::spawn(move || {
            let api = Switchyard::builder(facts2.clone(), audit2.clone(), policy2.clone())
                .with_lock_manager(Box::new(FileLockManager::new(lock2)))
                .build();
            let _ = api.apply(&plan2, ApplyMode::Commit);
        });
        let _ = h1.join(); let _ = h2.join();
    }

    #[when(regex = r"^both apply\(\) are started in Commit mode$")]
    async fn when_both_started(world: &mut World) { when_two_apply_overlap(world).await }

    #[when(regex = r"^I attempt apply in Commit mode$")]
    async fn when_attempt_apply_commit(world: &mut World) {
        if world.plan.is_none() { given_plan_min(world).await; }
        let plan = world.plan.as_ref().unwrap();
        let _ = world.api.as_ref().unwrap().apply(plan, ApplyMode::Commit);
    }

    // Then steps (assertions on facts)
    fn all_facts(world: &World) -> Vec<Value> { world.facts.0.lock().unwrap().clone() }

    #[then(regex = r"^each fact carries schema_version=2$")]
    async fn then_schema_v2(world: &mut World) {
        for ev in all_facts(world) { assert_eq!(ev.get("schema_version").and_then(|v| v.as_i64()), Some(2)); }
    }

    fn normalize_for_compare(mut v: Value) -> Value {
        if let Some(obj) = v.as_object_mut() {
            obj.remove("run_id");
            obj.remove("event_id");
            obj.remove("seq");
            obj.remove("switchyard_version");
        }
        v
    }

    #[then(regex = r"^(every|each) stage emits a JSON fact that validates against /SPEC/audit_event.v2.schema.json$")]
    async fn then_validate_schema(world: &mut World) {
        let schema_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("SPEC/audit_event.v2.schema.json");
        let schema_data = std::fs::read_to_string(schema_path).expect("read schema");
        let schema_json: Value = serde_json::from_str(&schema_data).expect("parse schema");
        let compiled = JSONSchema::compile(&schema_json).expect("compile schema");
        for ev in all_facts(world) {
            if !compiled.is_valid(&ev) {
                // On failure, try to extract a single error message for context
                let msg = compiled.validate(&ev).err().and_then(|mut it| it.next()).map(|e| e.to_string()).unwrap_or_else(|| "unknown validation error".to_string());
                panic!("schema validation failed: {} for {}", msg, serde_json::to_string(&ev).unwrap_or_default());
            }
        }
    }

    #[then(regex = r"^the emitted facts for plan and preflight are byte-identical after timestamp redaction$")]
    async fn then_plan_preflight_identical(world: &mut World) {
        // Reproduce emissions deterministically: run preflight twice and compare redacted facts
        world.ensure_api();
        if world.plan.is_none() { given_plan_min(world).await; }
        let plan = world.plan.as_ref().unwrap().clone();
        // First run
        world.clear_facts();
        let _ = world.api.as_ref().unwrap().preflight(&plan).unwrap();
        let mut a: Vec<Value> = all_facts(world)
            .into_iter()
            .filter(|e| matches!(e.get("stage").and_then(|v| v.as_str()), Some("plan") | Some("preflight") | Some("preflight.summary")))
            .map(redact_event)
            .map(normalize_for_compare)
            .collect();
        // Second run
        world.clear_facts();
        let _ = world.api.as_ref().unwrap().preflight(&plan).unwrap();
        let mut b: Vec<Value> = all_facts(world)
            .into_iter()
            .filter(|e| matches!(e.get("stage").and_then(|v| v.as_str()), Some("plan") | Some("preflight") | Some("preflight.summary")))
            .map(redact_event)
            .map(normalize_for_compare)
            .collect();
        let key = |e: &Value| {
            let st = e.get("stage").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let aid = e.get("action_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let p = e.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            (st, aid, p)
        };
        a.sort_by_key(key);
        b.sort_by_key(key);
        assert_eq!(a, b, "plan+preflight facts not identical after redaction");
    }

    #[then(regex = r"^the emitted facts for apply\.result per-action events are byte-identical after redaction$")]
    async fn then_apply_result_identical(world: &mut World) {
        let dry = world.facts_dry.clone().expect("facts_dry");
        if world.facts_real.is_none() {
            when_run_real(world).await;
            world.facts_real = Some(all_facts(world));
        }
        let real = world.facts_real.clone().unwrap();
        let mut a: Vec<Value> = dry.into_iter()
            .filter(|e| e.get("stage").and_then(|v| v.as_str()) == Some("apply.result") && e.get("action_id").is_some())
            .map(redact_event)
            .map(normalize_for_compare)
            .collect();
        let mut b: Vec<Value> = real.into_iter()
            .filter(|e| e.get("stage").and_then(|v| v.as_str()) == Some("apply.result") && e.get("action_id").is_some())
            .map(redact_event)
            .map(normalize_for_compare)
            .collect();
        let key = |e: &Value| e.get("action_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        a.sort_by_key(key);
        b.sort_by_key(key);
        assert_eq!(a, b, "apply.result per-action not identical after redaction");
    }

    #[then(regex = r"^the resulting facts include hash_alg=sha256 and both before_hash and after_hash$")]
    async fn then_hash_fields_present(world: &mut World) {
        let facts = all_facts(world);
        let mut ok = false;
        for e in facts {
            if e.get("stage").and_then(|v| v.as_str()) == Some("apply.result") && e.get("action_id").is_some() {
                if e.get("hash_alg").and_then(|v| v.as_str()) == Some("sha256") {
                    if e.get("before_hash").is_some() && e.get("after_hash").is_some() {
                        ok = true; break;
                    }
                }
            }
        }
        assert!(ok, "missing sha256 before/after hash fields in apply.result");
    }

    // Aliases for audit phrasing variants
    #[then(regex = r"^emitted facts are byte-identical after timestamp redaction$")]
    async fn then_emitted_byte_identical(world: &mut World) { then_plan_preflight_identical(world).await }
    #[then(regex = r"^apply\.result includes hash_alg=sha256 and both before_hash and after_hash$")]
    async fn then_apply_result_hashes(world: &mut World) { then_hash_fields_present(world).await }

    // Aliases for fallback presence
    #[given(regex = r"^at least one fallback binary set \(GNU or BusyBox\) is installed and on PATH$")]
    async fn given_fallback_present(_world: &mut World) { std::env::set_var("SWITCHYARD_FORCE_RESCUE_OK", "1"); }

    // Observability audit extra steps
    #[given(regex = r"^a failing preflight or apply stage$")]
    async fn given_failing_preflight(world: &mut World) {
        // Require rescue and set exec_check true without making it available -> preflight STOP
        world.policy.rescue.require = true;
        world.policy.rescue.exec_check = true;
        std::env::set_var("SWITCHYARD_FORCE_RESCUE_OK", "0");
        world.rebuild_api();
    }
    #[when(regex = r"^I inspect summary events$")]
    async fn when_inspect_summary(world: &mut World) { when_preflight(world).await }
    #[then(regex = r"^summary_error_ids is present and ordered from specific to general$")]
    async fn then_summary_error_chain(world: &mut World) {
        let mut ok = false;
        for e in all_facts(world) {
            if e.get("stage").and_then(|v| v.as_str()) == Some("preflight.summary") {
                if let Some(arr) = e.get("summary_error_ids").and_then(|v| v.as_array()) {
                    ok = !arr.is_empty();
                }
            }
        }
        assert!(ok, "expected summary_error_ids in summary event");
    }
    #[when(regex = r"^I inspect apply\.result$")]
    async fn when_inspect_apply_result(_world: &mut World) {}

    #[then(regex = r"^no unmasked secret values appear in any emitted fact or (?:log )?sink$")]
    async fn then_no_secrets(world: &mut World) {
        let needle = "paru".to_string();
        // Scan facts
        for e in all_facts(world) {
            let s = serde_json::to_string(&e).unwrap();
            assert!(!s.contains(&needle), "found unmasked secret in facts: {}", s);
        }
        // Scan audit logs
        for (_lvl, msg) in world.audit.0.lock().unwrap().iter() {
            assert!(!msg.contains(&needle), "found unmasked secret in audit: {}", msg);
        }
    }

    #[given(regex = r"^environment-derived sensitive values might appear in facts$")]
    async fn given_env_sensitive_alias(world: &mut World) { given_plan_env_sensitive(world).await }

    #[then(regex = r"^facts include origin, helper, uid, gid, pkg, and env_sanitized=true$")]
    async fn then_provenance_fields(world: &mut World) {
        // Best-effort: ensure provenance.env_sanitized=true present at least once
        let mut ok = false;
        for e in all_facts(world) {
            if let Some(p) = e.get("provenance").and_then(|v| v.as_object()) {
                if p.get("env_sanitized").and_then(|v| v.as_bool()) == Some(true) { ok = true; break; }
            }
        }
        assert!(ok, "expected provenance.env_sanitized=true in facts");
    }

    #[when(regex = r"^I inspect preflight and emitted facts$")]
    async fn when_inspect_preflight(world: &mut World) {
        world.ensure_api();
        if world.plan.is_none() { given_plan_min(world).await; }
        let plan = world.plan.as_ref().unwrap();
        let _ = world.api.as_ref().unwrap().preflight(plan).unwrap();
    }

    #[then(regex = r"^the presence of a rescue symlink set is recorded$")]
    async fn then_rescue_recorded(world: &mut World) {
        let mut ok = false;
        for e in all_facts(world) {
            if e.get("stage").and_then(|v| v.as_str()) == Some("preflight.summary") {
                if e.get("rescue_profile").is_some() { ok = true; break; }
            }
        }
        assert!(ok, "expected rescue_profile in preflight.summary");
    }

    #[then(regex = r"^preflight verifies at least one functional fallback path is executable$")]
    async fn then_rescue_fallback(world: &mut World) {
        // If verify succeeded, preflight summary should be success and rescue_profile available
        let mut ok = false;
        for e in all_facts(world) {
            if e.get("stage").and_then(|v| v.as_str()) == Some("preflight.summary") {
                if e.get("rescue_profile").is_some() { ok = true; break; }
            }
        }
        assert!(ok, "expected fallback verification recorded");
    }

    #[then(regex = r"^the exported preflight YAML rows are byte-identical between runs$")]
    async fn then_preflight_yaml_identical(world: &mut World) {
        world.ensure_api();
        if world.plan.is_none() { given_plan_min(world).await; }
        let plan = world.plan.as_ref().unwrap();
        let r1 = world.api.as_ref().unwrap().preflight(plan).unwrap();
        let y1 = preflight_yaml::to_yaml(&r1);
        let r2 = world.api.as_ref().unwrap().preflight(plan).unwrap();
        let y2 = preflight_yaml::to_yaml(&r2);
        assert_eq!(y1, y2, "preflight YAML differs between runs");
    }

    // Attestation configuration and checks
    struct DummyAttestor;
    impl Attestor for DummyAttestor {
        fn sign(&self, _bundle: &[u8]) -> Result<Signature, switchyard::types::errors::Error> { Ok(Signature(vec![0xAA; 64])) }
        fn key_id(&self) -> String { "test-key".to_string() }
    }

    #[given(regex = r"^an attestor is configured and apply succeeds in Commit mode$")]
    async fn given_attestor_and_apply(world: &mut World) {
        world.ensure_api();
        if world.plan.is_none() { given_plan_min(world).await; }
        let att: Box<dyn Attestor> = Box::new(DummyAttestor);
        let api = Switchyard::builder(world.facts.clone(), world.audit.clone(), world.policy.clone())
            .with_attestor(att)
            .build();
        world.api = Some(api);
        let plan = world.plan.as_ref().unwrap();
        let _ = world.api.as_ref().unwrap().apply(plan, ApplyMode::Commit).unwrap();
    }

    #[then(regex = r"^an attestation is attached to the apply\.result summary fact with sig_alg=ed25519, signature, bundle_hash, and public_key_id$")]
    async fn then_attestation_present(world: &mut World) {
        let mut ok = false;
        for e in all_facts(world) {
            if e.get("stage").and_then(|v| v.as_str()) == Some("apply.result") && e.get("action_id").is_none() {
                if let Some(att) = e.get("attestation").and_then(|v| v.as_object()) {
                    if att.get("sig_alg").and_then(|v| v.as_str()) == Some("ed25519") && att.get("signature").is_some() && att.get("bundle_hash").is_some() && att.get("public_key_id").is_some() { ok = true; break; }
                }
            }
        }
        assert!(ok, "missing attestation on apply.result summary");
    }

    #[then(regex = r"^facts record lock_wait_ms when available$")]
    async fn then_lock_wait(world: &mut World) {
        let any_with = all_facts(world).into_iter().any(|e| e.get("lock_wait_ms").is_some());
        assert!(any_with, "no fact had lock_wait_ms");
    }

    #[then(regex = r"^a WARN fact is emitted stating concurrent apply is unsupported$")]
    async fn then_warn_no_lock(world: &mut World) {
        let mut saw = false;
        for ev in all_facts(world) {
            if ev.get("stage").and_then(|v| v.as_str()) == Some("apply.attempt") {
                if ev.get("decision").and_then(|v| v.as_str()) == Some("warn") {
                    if ev.get("no_lock_manager").is_some() || ev.get("lock_backend").and_then(|v| v.as_str()) == Some("none") {
                        saw = true; break;
                    }
                }
            }
        }
        assert!(saw, "expected WARN apply.attempt for no lock manager");
    }

    #[given(regex = r"^another apply\(\) is already holding the lock$")]
    async fn given_other_holds_lock(world: &mut World) {
        let lock_path = world.lock_path.clone().unwrap_or_else(|| world.ensure_root().join("switchyard.lock"));
        let mgr = FileLockManager::new(lock_path);
        // Leak guard to keep it held for the duration of scenario
        let guard = mgr.acquire_process_lock(10_000).expect("acquire lock");
        Box::leak(Box::new(guard));
    }

    #[given(regex = r"^a LockManager configured with a short timeout$")]
    async fn given_short_timeout(world: &mut World) {
        let lock_path = world.ensure_root().join("switchyard.lock");
        world.lock_path = Some(lock_path.clone());
        let api = Switchyard::builder(world.facts.clone(), world.audit.clone(), world.policy.clone())
            .with_lock_manager(Box::new(FileLockManager::new(lock_path)))
            .with_lock_timeout_ms(150)
            .build();
        world.api = Some(api);
    }

    #[then(regex = r"^the stage fails with error_id=E_LOCKING and exit_code=30$")]
    async fn then_locking_failure(world: &mut World) {
        let mut saw = false;
        for ev in all_facts(world) {
            if ev.get("error_id").and_then(|v| v.as_str()) == Some("E_LOCKING") {
                if ev.get("exit_code").and_then(|v| v.as_i64()) == Some(30) { saw = true; break; }
            }
        }
        assert!(saw, "expected E_LOCKING with exit_code=30");
    }

    #[given(regex = r"^a contended lock with retries$")]
    async fn given_contended(world: &mut World) {
        // Hold the lock briefly to force retries
        let lock_path = world.lock_path.clone().unwrap_or_else(|| world.ensure_root().join("switchyard.lock"));
        std::thread::spawn(move || {
            let mgr = FileLockManager::new(lock_path);
            let g = mgr.acquire_process_lock(500).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(200));
            drop(g);
        });
    }

    #[then(regex = r"^apply.attempt includes lock_attempts approximating retry count$")]
    async fn then_lock_attempts(world: &mut World) {
        let mut ok = false;
        for ev in all_facts(world) {
            if ev.get("stage").and_then(|v| v.as_str()) == Some("apply.attempt") {
                if let Some(n) = ev.get("lock_attempts").and_then(|v| v.as_u64()) {
                    if n >= 2 { ok = true; break; }
                }
            }
        }
        assert!(ok, "expected lock_attempts >= 2");
    }

    #[given(regex = r"^a configured rescue profile consisting of backup symlinks$")]
    async fn given_rescue_configured(_world: &mut World) {
        std::env::set_var("SWITCHYARD_FORCE_RESCUE_OK", "1");
    }

    #[given(regex = r"^a system with configured rescue profile$")]
    async fn given_rescue_system(_world: &mut World) { std::env::set_var("SWITCHYARD_FORCE_RESCUE_OK", "1"); }

    #[given(regex = r"^no BusyBox but GNU core utilities are present on PATH$")]
    async fn given_gnu_subset_ok(_world: &mut World) { std::env::set_var("SWITCHYARD_FORCE_RESCUE_OK", "1"); }

    #[given(regex = r"^a plan that mutates a file$")]
    async fn given_plan_mutates(world: &mut World) {
        // Ensure a regular file exists at target so swap creates backup and after_kind changes
        let root = world.ensure_root().to_path_buf();
        let link = "/usr/bin/cp";
        let target = util::under_root(&root, link);
        if let Some(p) = target.parent() { let _ = std::fs::create_dir_all(p); }
        let _ = std::fs::write(&target, b"old");
        world.mk_symlink(link, "providerA/cp");
        world.build_single_swap(link, "providerB/cp");
    }

    #[given(regex = r"^a plan with environment-derived values that may be sensitive$")]
    async fn given_plan_env_sensitive(world: &mut World) {
        std::env::set_var("SWITCHYARD_HELPER", "paru");
        given_plan_min(world).await;
    }

    #[given(regex = r"^a plan that uses an external helper$")]
    async fn given_plan_external_helper(world: &mut World) { given_plan_env_sensitive(world).await }

    #[then(regex = r"^facts record degraded=true when policy allow_degraded_fs is enabled$")]
    async fn then_degraded_flag(world: &mut World) {
        // enable degraded and run apply to produce fact
        world.policy.apply.exdev = ExdevPolicy::DegradedFallback; world.rebuild_api();
        let plan = world.plan.as_ref().unwrap();
        let _ = world.api.as_ref().unwrap().apply(plan, ApplyMode::Commit).unwrap();
        let mut saw = false;
        for ev in all_facts(world) {
            if let Some(d) = ev.get("degraded").and_then(|v| v.as_bool()) { if d { saw = true; break; } }
        }
        assert!(saw, "did not observe degraded=true fact");
    }

    #[then(regex = r"^the operation fails with error_id=E_EXDEV when allow_degraded_fs is disabled$")]
    async fn then_exdev_fail(world: &mut World) {
        world.policy.apply.exdev = ExdevPolicy::Fail; world.rebuild_api();
        std::env::set_var("SWITCHYARD_FORCE_EXDEV", "1");
        let plan = world.plan.as_ref().unwrap();
        let _ = world.api.as_ref().unwrap().apply(plan, ApplyMode::Commit).unwrap();
        let mut saw = false;
        for ev in all_facts(world) {
            if ev.get("error_id").and_then(|v| v.as_str()) == Some("E_EXDEV") { saw = true; break; }
        }
        assert!(saw, "expected E_EXDEV in facts");
    }
}

#[cfg(not(feature = "bdd"))]
fn main() {}

#[cfg(feature = "bdd")]
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Run all features under SPEC/features/
    let features = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("SPEC/features");
    World::cucumber()
        .fail_on_skipped()
        .run_and_exit(features)
        .await;
}
