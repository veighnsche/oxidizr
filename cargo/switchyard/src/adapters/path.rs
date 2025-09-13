use crate::types::{errors::Result, safepath::SafePath};

pub trait PathResolver {
    /// Resolve a binary name to a `SafePath`.
    /// # Errors
    /// Returns an error if the binary cannot be resolved to a safe path.
    fn resolve(&self, bin: &str) -> Result<SafePath>;
}
