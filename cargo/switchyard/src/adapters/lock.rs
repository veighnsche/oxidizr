use crate::types::errors::Result;

pub trait LockGuard: Send {}

pub trait LockManager: Send + Sync {
    fn acquire_process_lock(&self, timeout_ms: u64) -> Result<Box<dyn LockGuard>>;
}
