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

/// Best-effort mapping from apply-stage error strings to a chain of stable summary error IDs.
/// Always includes a top-level classification; may include co-emitted categories like `E_OWNERSHIP`.
#[must_use]
pub fn infer_summary_error_ids(errors: &[String]) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    let joined = errors.join("; ").to_lowercase();
    if joined.contains("smoke") {
        out.push(id_str(ErrorId::E_SMOKE));
    }
    if joined.contains("lock") {
        out.push(id_str(ErrorId::E_LOCKING));
    }
    if joined.contains("ownership") {
        out.push(id_str(ErrorId::E_OWNERSHIP));
    }
    if joined.contains("exdev") {
        out.push(id_str(ErrorId::E_EXDEV));
    }
    if joined.contains("atomic") || joined.contains("symlink") {
        out.push(id_str(ErrorId::E_ATOMIC_SWAP));
    }
    if joined.contains("backup") && joined.contains("missing") {
        out.push(id_str(ErrorId::E_BACKUP_MISSING));
    }
    if joined.contains("restore") && joined.contains("failed") {
        out.push(id_str(ErrorId::E_RESTORE_FAILED));
    }
    if out.is_empty() {
        out.push(id_str(ErrorId::E_POLICY));
    } else {
        // Ensure E_POLICY is present last for routing when other specifics exist
        out.push(id_str(ErrorId::E_POLICY));
    }
    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    out.into_iter()
        .filter(|id| seen.insert(*id))
        .collect()
}

impl From<crate::types::errors::Error> for ApiError {
    fn from(e: crate::types::errors::Error) -> Self {
        use crate::types::errors::ErrorKind::{InvalidPath, Io, Policy};
        match e.kind {
            InvalidPath | Io => ApiError::FilesystemError(e.msg),
            Policy => ApiError::PolicyViolation(e.msg),
        }
    }
}

// Stable identifiers aligned with SPEC/error_codes.toml
// We intentionally keep SCREAMING_SNAKE_CASE to match emitted IDs.
#[allow(non_camel_case_types, reason = "Error IDs must match SPEC/error_codes.toml format")]
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

#[must_use]
pub const fn id_str(id: ErrorId) -> &'static str {
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

#[must_use]
pub const fn exit_code_for(id: ErrorId) -> i32 {
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

#[must_use]
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
