use crate::types::{safepath::SafePath, errors::Result};

#[derive(Clone, Debug)]
pub struct OwnershipInfo {
    pub uid: u32,
    pub gid: u32,
    pub pkg: String,
}

pub trait OwnershipOracle {
    fn owner_of(&self, path: &SafePath) -> Result<OwnershipInfo>;
}
