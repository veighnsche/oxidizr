// tests/helpers/env.rs
// Scoped environment variable guard for tests. Restores previous value on drop.

use std::env;

#[derive(Debug)]
pub struct ScopedEnv {
    key: String,
    prev: Option<String>,
}

impl ScopedEnv {
    pub fn set<K: Into<String>, V: Into<String>>(key: K, value: V) -> Self {
        let key_s = key.into();
        let prev = env::var(&key_s).ok();
        env::set_var(&key_s, value.into());
        ScopedEnv { key: key_s, prev }
    }
}

impl Drop for ScopedEnv {
    fn drop(&mut self) {
        match &self.prev {
            Some(v) => env::set_var(&self.key, v),
            None => env::remove_var(&self.key),
        }
    }
}
