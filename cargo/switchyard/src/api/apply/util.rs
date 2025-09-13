use crate::api::DebugLockManager;

pub(crate) fn lock_backend_label(mgr: Option<&dyn DebugLockManager>) -> String {
    if let Some(m) = mgr {
        // Best-effort dynamic type name; map common implementations to concise labels
        let tn = std::any::type_name_of_val(m);
        if tn.ends_with("::file::FileLockManager") || tn.ends_with("FileLockManager") {
            "file".to_string()
        } else {
            // Fallback: last segment lowercased
            tn.rsplit("::").next().unwrap_or("custom").to_lowercase()
        }
    } else {
        "none".to_string()
    }
}
