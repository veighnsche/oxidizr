use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("policy violation: {0}")]
    PolicyViolation(String),
    #[error("locking timeout: {0}")]
    LockingTimeout(String),
    #[error("filesystem error: {0}")]
    FilesystemError(String),
    #[error("cross-filesystem degraded path not allowed: {0}")]
    ExdevDegraded(String),
    #[error("smoke tests failed")]
    SmokeFailed,
    #[error("ownership check failed: {0}")]
    OwnershipError(String),
    #[error("attestation failed: {0}")]
    AttestationFailed(String),
}

impl From<crate::types::errors::Error> for ApiError {
    fn from(e: crate::types::errors::Error) -> Self {
        use crate::types::errors::ErrorKind::*;
        match e.kind {
            InvalidPath | Io => ApiError::FilesystemError(e.msg),
            Policy => ApiError::PolicyViolation(e.msg),
        }
    }
}
