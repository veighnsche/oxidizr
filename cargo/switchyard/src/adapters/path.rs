use crate::types::{errors::Result, safepath::SafePath};

pub trait PathResolver {
    fn resolve(&self, bin: &str) -> Result<SafePath>;
}
