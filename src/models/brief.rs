use serde::{Deserialize, Serialize};

/// Metadata about a generated brief, stored alongside it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BriefMeta {
    pub project: String,
    pub generated_at: String,
    /// The repo path this brief was generated for.
    pub repo: String,
    /// Number of sessions included in the brief.
    pub session_count: usize,
    /// ID of the most recent session included.
    pub latest_session_id: Option<String>,
}
