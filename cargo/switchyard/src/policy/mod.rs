//! Policy configuration, gating, and rescue verification.
//!
//! The `policy` module centralizes production hardening knobs and gating logic
//! used by preflight and apply stages. Consumers typically construct a
//! [`Policy`](crate::policy::Policy) via presets (`production_preset`,
//! `coreutils_switch_preset`) and then customize fields before creating a
//! [`Switchyard`](crate::Switchyard) instance.
//!
//! Submodules:
//! - `config`: policy struct and presets
//! - `gating`: apply-stage gating parity with preflight
//! - `rescue`: rescue toolset verification helpers
//!
//! The crate may expose compatibility re-exports at the top-level temporarily;
//! prefer importing from `switchyard::policy`.

pub mod config;
pub mod gating;
pub mod rescue;
pub mod types;

pub use config::Policy;
