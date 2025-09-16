#[cfg(feature = "debian-alternatives")]
pub mod alternatives;
pub mod debian;
pub mod debian_adapter;
#[cfg(feature = "debian-divert")]
pub mod divert;
pub mod preflight;
