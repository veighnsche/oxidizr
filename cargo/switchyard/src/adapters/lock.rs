use crate::types::errors::Result;

pub trait LockGuard {}

pub trait LockManager {
    fn acquire_process_lock(&self) -> Result<Box<dyn LockGuard>>;
}
