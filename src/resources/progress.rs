use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A Canvas background job progress object.
///
/// Poll [`Progress::completion`] until `workflow_state` is `"completed"` or `"failed"`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Progress {
    pub id: u64,
    pub context_id: Option<u64>,
    pub context_type: Option<String>,
    pub user_id: Option<u64>,
    pub tag: Option<String>,
    pub completion: Option<f64>,
    pub workflow_state: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub message: Option<String>,
    pub results: Option<serde_json::Value>,
    pub url: Option<String>,
}
