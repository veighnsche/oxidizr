use jsonschema::JSONSchema;
use serde_json::Value;
use std::path::Path;
use std::sync::OnceLock;

static SCHEMA_V2: OnceLock<JSONSchema> = OnceLock::new();

pub fn compiled_v2() -> &'static JSONSchema {
    SCHEMA_V2.get_or_init(|| {
        let schema_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("SPEC/audit_event.v2.schema.json");
        let schema_data = std::fs::read_to_string(schema_path).expect("read schema");
        let schema_json: Value = serde_json::from_str(&schema_data).expect("parse schema");
        JSONSchema::compile(&schema_json).expect("compile schema")
    })
}
