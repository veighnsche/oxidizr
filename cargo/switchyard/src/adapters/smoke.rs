use crate::types::plan::Plan;

#[derive(Debug)]
pub struct SmokeFailure;

pub trait SmokeTestRunner {
    fn run(&self, plan: &Plan) -> std::result::Result<(), SmokeFailure>;
}
