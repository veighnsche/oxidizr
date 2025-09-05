//! A library for abstracting over different coreutils implementations.
//! 
//! This library provides a unified interface for working with different
//! coreutils implementations, specifically GNU coreutils and uutils.

pub mod core;
pub mod error;
pub mod cli;

// Re-export commonly used items
pub use crate::core::{CoreUtil, CoreUtilsImpl, create_core_util};
pub use crate::error::{CoreutilsError, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_utils_creation() {
        let util = create_core_util("ls", CoreUtilsImpl::Gnu);
        assert_eq!(util.name(), "GNU Coreutils");
    }
}
