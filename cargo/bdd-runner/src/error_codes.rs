use std::collections::HashMap;

pub fn load_error_codes(path: &str) -> anyhow::Result<HashMap<String, i32>> {
    let s = std::fs::read_to_string(path)?;
    let _v: toml::Value = s.parse()?;
    // TODO: map into HashMap; for now we just ensure it parses
    Ok(HashMap::new())
}
