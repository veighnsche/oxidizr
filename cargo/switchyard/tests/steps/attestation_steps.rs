use cucumber::{given, then, when};

use crate::bdd_world::World;
use switchyard::adapters::{AttestationError, Attestor, Signature};
use switchyard::api::DebugAttestor;
use switchyard::types::plan::ApplyMode;

#[derive(Debug)]
struct DummyAttestor;
impl Attestor for DummyAttestor {
    fn sign(&self, _bundle: &[u8]) -> Result<Signature, AttestationError> {
        Ok(Signature(vec![0xAA; 64]))
    }
    fn key_id(&self) -> String {
        "test-key".to_string()
    }
}

#[when(regex = r"^I complete an apply$")]
pub async fn when_complete_apply(world: &mut World) {
    // Ensure we have a minimal plan and an API instance; allow unlocked commit for tests
    if world.plan.is_none() {
        crate::steps::plan_steps::given_plan_min(world).await;
    }
    world.policy.governance.allow_unlocked_commit = true;
    world.policy.apply.override_preflight = true;
    // Ensure an attestor is configured so the summary includes attestation fields
    let att: Box<dyn DebugAttestor> = Box::new(DummyAttestor);
    if world.api.is_none() {
        let api = switchyard::api::Switchyard::builder(
            world.facts.clone(),
            world.audit.clone(),
            world.policy.clone(),
        )
        .with_attestor(att)
        .build();
        world.api = Some(api);
    } else {
        // Rebuild to attach attestor while preserving policy/facts/audit
        let api = switchyard::api::Switchyard::builder(
            world.facts.clone(),
            world.audit.clone(),
            world.policy.clone(),
        )
        .with_attestor(att)
        .build();
        world.api = Some(api);
    }
    let plan = world.plan.as_ref().unwrap();
    let _ = world
        .api
        .as_ref()
        .unwrap()
        .apply(plan, ApplyMode::Commit)
        .unwrap();
}

#[given(regex = r"^an attestor is configured and apply succeeds in Commit mode$")]
pub async fn given_attestor_and_apply(world: &mut World) {
    // Allow commit without lock and bypass preflight gating to ensure apply proceeds
    world.policy.governance.allow_unlocked_commit = true;
    world.policy.apply.override_preflight = true;
    if world.plan.is_none() {
        crate::steps::plan_steps::given_plan_min(world).await;
    }
    let att: Box<dyn DebugAttestor> = Box::new(DummyAttestor);
    let api = switchyard::api::Switchyard::builder(
        world.facts.clone(),
        world.audit.clone(),
        world.policy.clone(),
    )
    .with_attestor(att)
    .build();
    world.api = Some(api);
    let plan = world.plan.as_ref().unwrap();
    let _ = world
        .api
        .as_ref()
        .unwrap()
        .apply(plan, ApplyMode::Commit)
        .unwrap();
}

#[then(
    regex = r"^an attestation is attached to the apply\.result summary fact with sig_alg=ed25519, signature, bundle_hash, and public_key_id$"
)]
pub async fn then_attestation_present(world: &mut World) {
    let mut ok = false;
    for e in world.all_facts() {
        if e.get("stage").and_then(|v| v.as_str()) == Some("apply.result")
            && e.get("action_id").is_none()
        {
            if let Some(att) = e.get("attestation").and_then(|v| v.as_object()) {
                if att.get("sig_alg").and_then(|v| v.as_str()) == Some("ed25519")
                    && att.get("signature").is_some()
                    && att.get("bundle_hash").is_some()
                    && att.get("public_key_id").is_some()
                {
                    ok = true;
                    break;
                }
            }
        }
    }
    assert!(ok, "missing attestation on apply.result summary");
}

#[then(
    regex = r"^attestation fields \(sig_alg, signature, bundle_hash, public_key_id\) are present$"
)]
pub async fn then_attestation_fields_alias(world: &mut World) {
    then_attestation_present(world).await
}
