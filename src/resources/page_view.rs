use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single Canvas page view (access log entry) for a user.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageView {
    pub id: Option<String>,
    pub app_name: Option<String>,
    pub url: Option<String>,
    pub context_type: Option<String>,
    pub asset_type: Option<String>,
    pub controller: Option<String>,
    pub action: Option<String>,
    pub contributed: Option<bool>,
    pub interaction_seconds: Option<f64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub developer_key_id: Option<u64>,
    pub user_request: Option<bool>,
    pub render_time: Option<f64>,
    pub user_agent: Option<String>,
    pub participated: Option<bool>,
    pub http_method: Option<String>,
    pub remote_ip: Option<String>,
}
