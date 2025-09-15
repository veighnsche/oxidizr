pub mod api_safety;
pub mod atomicity;
pub mod conservatism_ci;
pub mod determinism;
pub mod degraded_fs;
pub mod smoke;
pub mod locking_aliases;
pub mod error_taxonomy;
pub mod rescue_aliases;

// Re-exports for cross-module convenience
pub use atomicity::when_apply_plan_replaces_cp;
pub use conservatism_ci::then_side_effects_not_performed;
pub use degraded_fs::{then_apply_fails_exdev_50, then_emitted_degraded_true_reason};
pub use smoke::then_auto_rollback_occurs;
