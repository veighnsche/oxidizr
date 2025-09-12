/// Render a SPEC-aligned YAML sequence from a `PreflightReport` rows collection.
/// This exporter is intended for tests and artifacts and preserves only the
/// keys defined in SPEC/preflight.yaml.
pub fn to_yaml(report: &crate::types::report::PreflightReport) -> String {
    use serde_json::Value as J;
    use serde_yaml::Value as Y;
    let mut items: Vec<Y> = Vec::new();
    for row in &report.rows {
        let mut map = serde_yaml::Mapping::new();
        let get = |k: &str| row.get(k).cloned().unwrap_or(J::Null);
        let keys = [
            "action_id",
            "path",
            "current_kind",
            "planned_kind",
            "policy_ok",
            "provenance",
            "notes",
        ];
        for k in keys.iter() {
            let v = get(k);
            if !v.is_null() {
                let y: Y = serde_yaml::to_value(v).unwrap_or(Y::Null);
                map.insert(Y::String((*k).to_string()), y);
            }
        }
        items.push(Y::Mapping(map));
    }
    serde_yaml::to_string(&Y::Sequence(items)).unwrap_or_else(|_| "[]\n".to_string())
}
