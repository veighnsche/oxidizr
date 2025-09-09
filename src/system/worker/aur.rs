use crate::error::Result;
use crate::logging::{audit_event_fields, AuditFields};
use which::which;

impl super::Worker {
    /// Get available AUR helper name if any
    pub fn aur_helper_name(&self) -> Result<Option<String>> {
        let candidates = self.aur_helper_candidates();
        for h in &candidates {
            if which(h).is_ok() {
                let _ = audit_event_fields(
                    "worker",
                    "aur_helper_name",
                    "found",
                    &AuditFields {
                        target: Some(h.to_string()),
                        ..Default::default()
                    },
                );
                return Ok(Some(h.to_string()));
            }
        }
        let _ = audit_event_fields(
            "worker",
            "aur_helper_name",
            "not_found",
            &AuditFields {
                target: if self.aur_helper.is_empty() {
                    None
                } else {
                    Some(self.aur_helper.clone())
                },
                artifacts: Some(candidates.iter().map(|s| s.to_string()).collect()),
                ..Default::default()
            },
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
