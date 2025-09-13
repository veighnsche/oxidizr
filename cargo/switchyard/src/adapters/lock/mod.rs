pub mod file;
use crate::types::errors::Result;

pub trait LockGuard: Send {}

pub trait LockManager: Send + Sync {
    /// Acquire a process lock with the specified timeout.
    /// # Errors
    /// Returns an error if the lock cannot be acquired within the timeout period.
    fn acquire_process_lock(&self, timeout_ms: u64) -> Result<Box<dyn LockGuard>>;
}
