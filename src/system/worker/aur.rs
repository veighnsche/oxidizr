use crate::error::Result;
use crate::logging::audit_event;
use which::which;

impl super::Worker {
    /// Get available AUR helper name if any
    pub fn aur_helper_name(&self) -> Result<Option<String>> {
        let candidates = self.aur_helper_candidates();
        for h in &candidates {
            if which(h).is_ok() {
                let _ = audit_event("worker", "aur_helper_name", "found", h, "", None);
                return Ok(Some(h.to_string()));
            }
        }
        let _ = audit_event(
            "worker",
            "aur_helper_name",
            "not_found",
            &self.aur_helper,
            "",
            None,
        );
        Ok(None)
    }

    pub(super) fn aur_helper_candidates(&self) -> Vec<&str> {
        if !self.aur_helper.is_empty() && self.aur_helper != "auto" && self.aur_helper != "none" {
            vec![self.aur_helper.as_str(), "paru", "yay", "trizen", "pamac"]
        } else {
            vec!["paru", "yay", "trizen", "pamac"]
        }
    }
}
