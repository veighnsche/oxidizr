// tests/helpers/lockmgr.rs
// A simple in-process lock manager for tests. Uses a process-global Mutex to serialize
// mutators with a bounded timeout.

use std::sync::{Mutex, TryLockError};
use std::time::{Duration, Instant};

use switchyard::adapters::{LockGuard, LockManager};
use switchyard::types::errors::{Error, ErrorKind, Result};

static GLOBAL: Mutex<()> = Mutex::new(());

#[derive(Debug, Default)]
pub struct TestLockManager;

struct Guard(std::sync::MutexGuard<'static, ()>);
impl Drop for Guard { fn drop(&mut self) { /* unlocked by drop of guard */ } }
impl LockGuard for Guard {}

impl TestLockManager {
    pub fn new() -> Self { Self }
}

impl LockManager for TestLockManager {
    fn acquire_process_lock(&self, timeout_ms: u64) -> Result<Box<dyn LockGuard>> {
        let t0 = Instant::now();
        loop {
            match GLOBAL.try_lock() {
                Ok(g) => return Ok(Box::new(Guard(g))),
                Err(TryLockError::WouldBlock) => {
                    if t0.elapsed() >= Duration::from_millis(timeout_ms) {
                        return Err(Error { kind: ErrorKind::Policy, msg: "E_LOCKING: timeout acquiring test lock".to_string() });
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(TryLockError::Poisoned(_)) => {
                    // Recover by retrying
                    if t0.elapsed() >= Duration::from_millis(timeout_ms) {
                        return Err(Error { kind: ErrorKind::Policy, msg: "E_LOCKING: timeout acquiring test lock (poison)".to_string() });
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_manager_basic() {
        let mgr = TestLockManager::new();
        let _g = mgr.acquire_process_lock(50).unwrap();
    }
}
