use serde::Serialize;

/// Typed representation of a preflight diff row.
/// Serialized to JSON for emission and report rows.
#[derive(Clone, Debug, Serialize)]
pub struct PreflightRow {
    /// Unique identifier for the action
    pub action_id: String,
    /// Path being checked
    pub path: String,
    /// Current kind of the path
    pub current_kind: String,
    /// Planned kind of the path
    pub planned_kind: String,
    /// Whether the policy check passed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_ok: Option<bool>,
    /// Provenance information for the path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<serde_json::Value>,
    /// Additional notes about the path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<String>>,
    /// Preservation information for the path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preservation: Option<serde_json::Value>,
    /// Whether preservation is supported for this path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preservation_supported: Option<bool>,
    /// Whether the path is ready for restore
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restore_ready: Option<bool>,
    /// Backup tag for the path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_tag: Option<String>,
}
