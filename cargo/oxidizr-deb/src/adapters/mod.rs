pub mod debian;
#[cfg(feature = "debian-alternatives")]
pub mod alternatives;
#[cfg(feature = "debian-divert")]
pub mod divert;
pub mod preflight;
