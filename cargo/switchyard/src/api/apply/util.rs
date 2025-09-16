use crate::api::DebugLockManager;

pub(crate) fn lock_backend_label(mgr: Option<&dyn DebugLockManager>) -> String {
    if let Some(m) = mgr {
        // MSRV-friendly detection: use Debug representation to hint at backend
        let dbg = format!("{:?}", m);
        if dbg.contains("FileLockManager") {
            "file".to_string()
        } else {
            "custom".to_string()
        }
    } else {
        "none".to_string()
    }
}
