use log::Level;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use switchyard::logging::{AuditSink, FactsEmitter};

#[derive(Debug, Clone, Default)]
pub struct CollectingEmitter(pub Arc<Mutex<Vec<Value>>>);

impl FactsEmitter for CollectingEmitter {
    fn emit(&self, _subsystem: &str, _event: &str, _decision: &str, fields: Value) {
        self.0.lock().unwrap().push(fields);
    }
}

#[derive(Debug, Clone, Default)]
pub struct CollectingAudit(pub Arc<Mutex<Vec<(Level, String)>>>);

impl AuditSink for CollectingAudit {
    fn log(&self, level: Level, msg: &str) {
        self.0.lock().unwrap().push((level, msg.to_string()));
    }
}

/// Utility helpers for tests
pub mod util {
    use std::path::{Path, PathBuf};

    use switchyard::types::safepath::SafePath;

    /// Convert a human path like "/usr/bin/ls" or "providerA/ls" into a path under `root`.
    pub fn under_root(root: &Path, p: &str) -> PathBuf {
        let trimmed = p.trim();
        if let Some(rel) = trimmed.strip_prefix('/') {
            root.join(rel)
        } else {
            root.join(trimmed)
        }
    }

    pub fn sp(root: &Path, p: &str) -> SafePath {
        let abs = under_root(root, p);
        SafePath::from_rooted(root, &abs).expect("SafePath from rooted")
    }
}
