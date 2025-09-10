pub fn validate_event_against_schema(_event: &serde_json::Value, _schema_path: &str) -> anyhow::Result<()> {
    // TODO: add JSON-schema validation (e.g., jsonschema crate)
    Ok(())
}
