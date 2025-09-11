use crate::types::plan::Plan;

#[derive(Debug)]
pub struct SmokeFailure;

pub trait SmokeTestRunner: Send + Sync {
    fn run(&self, plan: &Plan) -> std::result::Result<(), SmokeFailure>;
}
