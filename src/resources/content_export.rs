use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContentExport {
    pub id: u64,
    pub created_at: Option<String>,
    pub export_type: Option<String>,
    pub user_id: Option<u64>,
    pub workflow_state: Option<String>,
    pub course_id: Option<u64>,
    pub attachment: Option<serde_json::Value>,
    pub progress_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ContentExportParams {
    pub export_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_notifications: Option<bool>,
}
