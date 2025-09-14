// tests/helpers/lockmgr.rs
// A simple in-process lock manager for tests. Uses a process-global Mutex to serialize
// mutators with a bounded timeout.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use switchyard::adapters::{LockGuard, LockManager};
use switchyard::types::errors::{Error, ErrorKind, Result};

static GLOBAL: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Default)]
pub struct TestLockManager;

struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        // Release the global spinlock
        GLOBAL.store(false, Ordering::Release);
    }
}
impl LockGuard for Guard {}

impl TestLockManager {
    pub fn new() -> Self {
        Self
    }
}

impl LockManager for TestLockManager {
    fn acquire_process_lock(&self, timeout_ms: u64) -> Result<Box<dyn LockGuard>> {
        let t0 = Instant::now();
        loop {
            if GLOBAL
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(Box::new(Guard));
            }
            if t0.elapsed() >= Duration::from_millis(timeout_ms) {
                return Err(Error {
                    kind: ErrorKind::Policy,
                    msg: "E_LOCKING: timeout acquiring test lock".to_string(),
                });
            }
            std::thread::sleep(Duration::from_millis(5));
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
