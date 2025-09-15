//! Per-instance simulation overrides used for test-only or controlled scenarios.

/// Overrides for simulation in tests and controlled environments.
#[derive(Clone, Debug, Default, Copy)]
pub struct Overrides {
    /// Force EXDEV behavior when performing atomic rename of symlinks.
    /// `Some(true)` simulates a cross-filesystem rename error to exercise degraded fallback paths.
    pub force_exdev: Option<bool>,
    /// Force rescue verification to succeed regardless of actual PATH state.
    pub force_rescue_ok: Option<bool>,
}

impl Overrides {
    #[must_use]
    /// Construct an overrides struct with `force_exdev` set.
    pub fn exdev(v: bool) -> Self {
        Self {
            force_exdev: Some(v),
            ..Default::default()
        }
    }
    #[must_use]
    /// Construct an overrides struct with `force_rescue_ok` set.
    pub fn rescue_ok(v: bool) -> Self {
        Self {
            force_rescue_ok: Some(v),
            ..Default::default()
        }
    }
}
