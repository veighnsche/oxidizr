pub mod fs;
use crate::types::{errors::Result, safepath::SafePath, OwnershipInfo};

pub trait OwnershipOracle: Send + Sync {
    fn owner_of(&self, path: &SafePath) -> Result<OwnershipInfo>;
}
