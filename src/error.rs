use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreutilsError {
    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid implementation: {0}")]
    InvalidImplementation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported distribution or release: {0}")]
    Incompatible(String),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, CoreutilsError>;
