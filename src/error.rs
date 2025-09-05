use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreutilsError {
    #[error("Command not found: {0}")]
    CommandNotFound(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Invalid implementation: {0}")]
    InvalidImplementation(String),
}

pub type Result<T> = std::result::Result<T, CoreutilsError>;
