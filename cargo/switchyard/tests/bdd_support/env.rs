use std::env;

#[derive(Debug)]
pub struct EnvGuard {
    key: String,
    previous: Option<String>,
}

impl EnvGuard {
    pub fn new<K: Into<String>, V: Into<String>>(key: K, value: V) -> Self {
        let key_s = key.into();
        let prev = env::var(&key_s).ok();
        env::set_var(&key_s, value.into());
        EnvGuard { key: key_s, previous: prev }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(v) => env::set_var(&self.key, v),
            None => env::remove_var(&self.key),
        }
    }
}

pub fn set_var_scoped<K: Into<String>, V: Into<String>>(key: K, value: V) -> EnvGuard {
    EnvGuard::new(key, value)
}
