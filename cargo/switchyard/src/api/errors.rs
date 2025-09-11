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

// Stable identifiers aligned with SPEC/error_codes.toml
// We intentionally keep SCREAMING_SNAKE_CASE to match emitted IDs.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum ErrorId {
    E_POLICY,
    E_OWNERSHIP,
    E_LOCKING,
    E_ATOMIC_SWAP,
    E_EXDEV,
    E_BACKUP_MISSING,
    E_RESTORE_FAILED,
    E_SMOKE,
    E_GENERIC,
}

pub fn id_str(id: ErrorId) -> &'static str {
    match id {
        ErrorId::E_POLICY => "E_POLICY",
        ErrorId::E_OWNERSHIP => "E_OWNERSHIP",
        ErrorId::E_LOCKING => "E_LOCKING",
        ErrorId::E_ATOMIC_SWAP => "E_ATOMIC_SWAP",
        ErrorId::E_EXDEV => "E_EXDEV",
        ErrorId::E_BACKUP_MISSING => "E_BACKUP_MISSING",
        ErrorId::E_RESTORE_FAILED => "E_RESTORE_FAILED",
        ErrorId::E_SMOKE => "E_SMOKE",
        ErrorId::E_GENERIC => "E_GENERIC",
    }
}

pub fn exit_code_for(id: ErrorId) -> i32 {
    match id {
        ErrorId::E_POLICY => 10,
        ErrorId::E_OWNERSHIP => 20,
        ErrorId::E_LOCKING => 30,
        ErrorId::E_ATOMIC_SWAP => 40,
        ErrorId::E_EXDEV => 50,
        ErrorId::E_BACKUP_MISSING => 60,
        ErrorId::E_RESTORE_FAILED => 70,
        ErrorId::E_SMOKE => 80,
        ErrorId::E_GENERIC => 1,
    }
}

pub fn exit_code_for_id_str(s: &str) -> Option<i32> {
    match s {
        "E_POLICY" => Some(10),
        "E_OWNERSHIP" => Some(20),
        "E_LOCKING" => Some(30),
        "E_ATOMIC_SWAP" => Some(40),
        "E_EXDEV" => Some(50),
        "E_BACKUP_MISSING" => Some(60),
        "E_RESTORE_FAILED" => Some(70),
        "E_SMOKE" => Some(80),
        _ => None,
    }
}
