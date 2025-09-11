use crate::types::{safepath::SafePath, errors::Result};

pub trait PathResolver {
    fn resolve(&self, bin: &str) -> Result<SafePath>;
}
