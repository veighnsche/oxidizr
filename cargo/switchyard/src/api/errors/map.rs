use std::io::ErrorKind;

use super::ErrorId;

/// Map errors from atomic swap/symlink replacement to stable `ErrorId`.
#[must_use]
pub fn map_swap_error(e: &std::io::Error) -> ErrorId {
    let emsg = e.to_string();
    if emsg.contains("sidecar write failed") {
        return ErrorId::E_POLICY;
    }
    match e.raw_os_error() {
        Some(code) if code == libc::EXDEV => ErrorId::E_EXDEV,
        _ => ErrorId::E_ATOMIC_SWAP,
    }
}

/// Map restore error kinds to stable `ErrorId` for telemetry.
#[must_use]
pub const fn map_restore_error_kind(kind: ErrorKind) -> ErrorId {
    match kind {
        ErrorKind::NotFound => ErrorId::E_BACKUP_MISSING,
        ErrorKind::PermissionDenied
        | ErrorKind::ConnectionRefused
        | ErrorKind::ConnectionReset
        | ErrorKind::HostUnreachable
        | ErrorKind::NetworkUnreachable
        | ErrorKind::ConnectionAborted
        | ErrorKind::NotConnected
        | ErrorKind::AddrInUse
        | ErrorKind::AddrNotAvailable
        | ErrorKind::NetworkDown
        | ErrorKind::BrokenPipe
        | ErrorKind::AlreadyExists
        | ErrorKind::WouldBlock
        | ErrorKind::NotADirectory
        | ErrorKind::IsADirectory
        | ErrorKind::DirectoryNotEmpty
        | ErrorKind::ReadOnlyFilesystem
        | ErrorKind::StaleNetworkFileHandle
        | ErrorKind::InvalidInput
        | ErrorKind::InvalidData
        | ErrorKind::TimedOut
        | ErrorKind::WriteZero
        | ErrorKind::StorageFull
        | ErrorKind::NotSeekable
        | ErrorKind::QuotaExceeded
        | ErrorKind::FileTooLarge
        | ErrorKind::ResourceBusy
        | ErrorKind::ExecutableFileBusy
        | ErrorKind::Deadlock
        | ErrorKind::CrossesDevices
        | ErrorKind::TooManyLinks
        | ErrorKind::InvalidFilename
        | ErrorKind::ArgumentListTooLong
        | ErrorKind::Interrupted
        | ErrorKind::Unsupported
        | ErrorKind::UnexpectedEof
        | ErrorKind::OutOfMemory
        | ErrorKind::Other
        | _ => ErrorId::E_RESTORE_FAILED,
    }
}
