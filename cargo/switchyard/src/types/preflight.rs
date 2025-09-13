use serde::Serialize;

/// Typed representation of a preflight diff row.
/// Serialized to JSON for emission and report rows.
#[derive(Clone, Debug, Serialize)]
pub struct PreflightRow {
    pub action_id: String,
    pub path: String,
    pub current_kind: String,
    pub planned_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_ok: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preservation: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preservation_supported: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restore_ready: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_tag: Option<String>,
}
