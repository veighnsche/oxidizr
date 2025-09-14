// compile-fail: constructing PlanInput with raw PathBuf instead of SafePath
use switchyard::types::{PlanInput, LinkRequest};

fn main() {
    let _input = PlanInput {
        link: vec![LinkRequest {
            // These should be switchyard::types::SafePath, not PathBuf
            source: std::path::PathBuf::from("/a"),
            target: std::path::PathBuf::from("/b"),
        }],
        restore: vec![],
    };
}
