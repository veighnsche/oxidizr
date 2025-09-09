use thiserror::Error;

/// Core error type for oxidizr-arch operations
#[derive(Debug, Error)]
pub enum Error {
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

    // New typed errors for enumerated exit codes / behavior
    /// No applets discovered to link after ensuring provider installation
    #[error("nothing to link: {0}")]
    NothingToLink(String),

    /// Restore failed because a backup file is missing (unless forced best-effort)
    #[error("restore backup missing for: {0}")]
    RestoreBackupMissing(String),

    /// Repository gating failed for a package with details
    #[error("repo gate failed for '{package}': {details}")]
    RepoGateFailed { package: String, details: String },

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
