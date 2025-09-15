use cucumber::then;

use crate::bdd_world::World;

#[then(regex = r"^preflight verifies a functional fallback path$")]
pub async fn then_rescue_verify(world: &mut World) {
    crate::steps::rescue_steps::then_rescue_fallback(world).await;
}

#[then(regex = r"^a rescue profile remains available for recovery$")]
pub async fn then_rescue_profile_available(world: &mut World) {
    crate::steps::rescue_steps::then_rescue_recorded(world).await;
}
