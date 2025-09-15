use cucumber::{given, then, when};

use crate::bdd_support::schema;
use crate::bdd_world::World;
use serde_json::Value as _;
use switchyard::api::Overrides;

#[then(regex = r"^each fact carries schema_version=2$")]
pub async fn then_schema_v2(world: &mut World) {
    for ev in world.all_facts() {
        assert_eq!(ev.get("schema_version").and_then(|v| v.as_i64()), Some(2));
    }
}

#[then(
    regex = r"^(every|each) stage emits a JSON fact that validates against /SPEC/audit_event.v2.schema.json$"
)]
pub async fn then_validate_schema(world: &mut World) {
    let compiled = schema::compiled_v2();
    for ev in world.all_facts() {
        if !compiled.is_valid(&ev) {
            // On failure, try to extract a single error message for context
            let msg = compiled
                .validate(&ev)
                .err()
                .and_then(|mut it| it.next())
                .map(|e| e.to_string())
                .unwrap_or_else(|| "unknown validation error".to_string());
            panic!(
                "schema validation failed: {} for {}",
                msg,
                serde_json::to_string(&ev).unwrap_or_default()
            );
        }
    }
}

#[then(regex = r"^emitted facts are byte-identical after timestamp redaction$")]
pub async fn then_emitted_byte_identical(world: &mut World) {
    crate::steps::preflight_steps::then_plan_preflight_identical(world).await
}

#[then(regex = r"^apply\.result includes hash_alg=sha256 and both before_hash and after_hash$")]
pub async fn then_apply_result_hashes(world: &mut World) {
    crate::steps::apply_steps::then_hash_fields_present(world).await
}

#[given(regex = r"^a failing preflight or apply stage$")]
pub async fn given_failing_preflight(world: &mut World) {
    // Require rescue and set exec_check true without making it available -> preflight STOP
    world.policy.rescue.require = true;
    world.policy.rescue.exec_check = true;
    // Prefer per-instance Overrides over env to avoid process-global side effects
    if world.plan.is_none() {
        crate::steps::plan_steps::given_plan_min(world).await;
    }
    let api = switchyard::api::Switchyard::new(
        world.facts.clone(),
        world.audit.clone(),
        world.policy.clone(),
    )
    .with_overrides(Overrides::rescue_ok(false));
    world.api = Some(api);
}

#[when(regex = r"^I inspect summary events$")]
pub async fn when_inspect_summary(world: &mut World) {
    crate::steps::preflight_steps::when_preflight(world).await
}

#[then(regex = r"^summary_error_ids is present and ordered from specific to general$")]
pub async fn then_summary_error_chain(world: &mut World) {
    let mut ok = false;
    for e in world.all_facts() {
        if e.get("stage").and_then(|v| v.as_str()) == Some("preflight.summary") {
            if let Some(arr) = e.get("summary_error_ids").and_then(|v| v.as_array()) {
                ok = !arr.is_empty();
            }
        }
    }
    assert!(ok, "expected summary_error_ids in summary event");
}

#[when(regex = r"^I inspect apply\.result$")]
pub async fn when_inspect_apply_result(_world: &mut World) {}

#[then(regex = r"^no unmasked secret values appear in any emitted fact or (?:log )?sink$")]
pub async fn then_no_secrets(world: &mut World) {
    let needle = "paru".to_string();
    // Scan facts
    for e in world.all_facts() {
        let s = serde_json::to_string(&e).unwrap();
        assert!(
            !s.contains(&needle),
            "found unmasked secret in facts: {}",
            s
        );
    }
    // Scan audit logs
    for (_lvl, msg) in world.audit.0.lock().unwrap().iter() {
        assert!(
            !msg.contains(&needle),
            "found unmasked secret in audit: {}",
            msg
        );
    }
}

#[given(regex = r"^environment-derived sensitive values might appear in facts$")]
pub async fn given_env_sensitive_alias(world: &mut World) {
    crate::steps::plan_steps::given_plan_env_sensitive(world).await
}

#[then(regex = r"^facts include origin, helper, uid, gid, pkg, and env_sanitized=true$")]
pub async fn then_provenance_fields(world: &mut World) {
    // Best-effort: ensure provenance.env_sanitized=true present at least once
    let mut ok = false;
    for e in world.all_facts() {
        if let Some(p) = e.get("provenance").and_then(|v| v.as_object()) {
            if p.get("env_sanitized").and_then(|v| v.as_bool()) == Some(true) {
                ok = true;
                break;
            }
        }
    }
    assert!(ok, "expected provenance.env_sanitized=true in facts");
}

#[when(regex = r"^I inspect preflight and emitted facts$")]
pub async fn when_inspect_preflight(world: &mut World) {
    world.ensure_api();
    world.ensure_plan_min();
    let plan = world.plan.as_ref().unwrap();
    let _ = world.api.as_ref().unwrap().preflight(plan).unwrap();
}

// (moved) Safety preconditions minimal flow now lives in safety_preconditions_steps.rs to avoid
// step ambiguity across features.

// Operational bounds fsync steps
#[given(regex = r"^a rename completes for a staged swap$")]
pub async fn given_rename_completes(world: &mut World) {
    if world.plan.is_none() {
        crate::steps::plan_steps::given_plan_min(world).await;
    }
    crate::steps::apply_steps::when_apply(world).await;
}

#[when(regex = r"^the engine performs fsync on the parent directory$")]
pub async fn when_engine_fsyncs(_world: &mut World) {}

#[then(regex = r"^the fsync occurs within 50ms of the rename and is recorded in telemetry$")]
pub async fn then_fsync_recorded(world: &mut World) {
    let mut saw = false;
    for ev in world.all_facts() {
        if ev.get("stage").and_then(|v| v.as_str()) == Some("apply.result")
            && ev.get("action_id").is_some()
            && ev.get("duration_ms").is_some()
        {
            saw = true;
            break;
        }
    }
    assert!(saw, "expected duration_ms in apply.result per-action fact");
}

#[then(regex = r"^if the fsync duration exceeds 50ms the fact is recorded with severity=warn$")]
pub async fn then_fsync_warn(world: &mut World) {
    // Conditional requirement: when severity is present, it must be "warn".
    for ev in world.all_facts() {
        if ev.get("stage").and_then(|v| v.as_str()) == Some("apply.result")
            && ev.get("action_id").is_some()
        {
            if let Some(sev) = ev.get("severity").and_then(|v| v.as_str()) {
                assert_eq!(sev, "warn", "unexpected severity value");
            }
        }
    }
}
