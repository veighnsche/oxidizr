// src/api/overrides.rs
// Per-instance simulation overrides used for test-only or controlled scenarios.

#[derive(Clone, Debug, Default, Copy)]
pub struct Overrides {
    pub force_exdev: Option<bool>,
    pub force_rescue_ok: Option<bool>,
}

impl Overrides {
    #[must_use]
    pub fn exdev(v: bool) -> Self {
        Self {
            force_exdev: Some(v),
            ..Default::default()
        }
    }
    #[must_use]
    pub fn rescue_ok(v: bool) -> Self {
        Self {
            force_rescue_ok: Some(v),
            ..Default::default()
        }
    }
}
