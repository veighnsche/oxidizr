pub mod fs;
use crate::types::{errors::Result, safepath::SafePath};

#[derive(Clone, Debug)]
pub struct OwnershipInfo {
    pub uid: u32,
    pub gid: u32,
    pub pkg: String,
}

pub trait OwnershipOracle: Send + Sync {
    fn owner_of(&self, path: &SafePath) -> Result<OwnershipInfo>;
}
