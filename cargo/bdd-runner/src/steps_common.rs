use cucumber::{given, then, when};
use crate::world::TestWorld;

// Generic catch-all step definitions to confirm runner wiring.
// They simply log the step text; replace with specific implementations later.

#[given(regex = r"^(.*)$")]
pub async fn any_given(_w: &mut TestWorld, text: String) {
    println!("TODO Given: {}", text);
}

#[when(regex = r"^(.*)$")]
pub async fn any_when(_w: &mut TestWorld, text: String) {
    println!("TODO When: {}", text);
}

#[then(regex = r"^(.*)$")]
pub async fn any_then(_w: &mut TestWorld, text: String) {
    println!("TODO Then: {}", text);
}
