use cucumber::{given, then, when};
use crate::world::TestWorld;

// Generic catch-all step definitions to confirm runner wiring.
// They simply log the step text; replace with specific implementations later.

fn maybe_fail_on_stub(kind: &str, text: &str) {
    let fail = std::env::var("BDD_FAIL_ON_STUB")
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "True"))
        .unwrap_or(false);
    if fail {
        panic!("Unimplemented {} step matched generic stub: {}", kind, text);
    }
}

#[given(regex = r"^(.*)$")]
pub async fn any_given(_w: &mut TestWorld, text: String) {
    println!("TODO Given: {}", text);
    maybe_fail_on_stub("Given", &text);
}

#[when(regex = r"^(.*)$")]
pub async fn any_when(_w: &mut TestWorld, text: String) {
    println!("TODO When: {}", text);
    maybe_fail_on_stub("When", &text);
}

#[then(regex = r"^(.*)$")]
pub async fn any_then(_w: &mut TestWorld, text: String) {
    println!("TODO Then: {}", text);
    maybe_fail_on_stub("Then", &text);
}
