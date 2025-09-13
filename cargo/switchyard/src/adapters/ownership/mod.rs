pub mod fs;
use crate::types::{errors::Result, safepath::SafePath, OwnershipInfo};

pub trait OwnershipOracle: Send + Sync {
    /// Get ownership information for the specified path.
    /// # Errors
    /// Returns an error if ownership information cannot be determined.
    fn owner_of(&self, path: &SafePath) -> Result<OwnershipInfo>;
}
