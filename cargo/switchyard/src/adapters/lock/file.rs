use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use crate::constants::LOCK_POLL_MS;
use crate::types::errors::{Error, ErrorKind, Result};
use fs2::FileExt;

use super::{LockGuard, LockManager};

#[derive(Debug)]
pub struct FileLockManager {
    path: PathBuf,
}

impl FileLockManager {
    #[must_use]
    pub const fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

struct FileGuard {
    file: File,
}

impl Drop for FileGuard {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

impl LockGuard for FileGuard {}

impl LockManager for FileLockManager {
    fn acquire_process_lock(&self, timeout_ms: u64) -> Result<Box<dyn LockGuard>> {
        let t0 = Instant::now();
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|e| Error {
                kind: ErrorKind::Io,
                msg: e.to_string(),
            })?;
        loop {
            match file.try_lock_exclusive() {
                Ok(()) => return Ok(Box::new(FileGuard { file })),
                Err(_e) => {
                    if t0.elapsed() >= Duration::from_millis(timeout_ms) {
                        return Err(Error {
                            kind: ErrorKind::Policy,
                            msg: "E_LOCKING: timeout acquiring process lock".to_string(),
                        });
                    }
                    thread::sleep(Duration::from_millis(LOCK_POLL_MS));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn file_lock_manager_timeout_and_success() {
        let td = tempfile::tempdir().unwrap();
        let lock_path = td.path().join("switchyard.lock");
        let mgr = FileLockManager::new(lock_path.clone());

        // Acquire lock in main thread
        let g = mgr.acquire_process_lock(200).expect("first lock");

        // Spawn a thread that tries to acquire and should timeout quickly
        let barrier = Arc::new(Barrier::new(2));
        let b2 = barrier.clone();
        let p2 = lock_path.clone();
        let h = thread::spawn(move || {
            let mgr2 = FileLockManager::new(p2);
            b2.wait();
            let res = mgr2.acquire_process_lock(150);
            assert!(res.is_err(), "second acquire should timeout");
        });
        barrier.wait();
        h.join().unwrap();

        // Drop first guard, new acquire should succeed
        drop(g);
        let g2 = mgr.acquire_process_lock(200).expect("lock after release");
        drop(g2);
    }
}
