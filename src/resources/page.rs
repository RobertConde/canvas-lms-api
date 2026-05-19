use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Page {
    pub page_id: Option<u64>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub hide_from_students: Option<bool>,
    pub editing_roles: Option<String>,
    pub last_edited_by: Option<serde_json::Value>,
    pub body: Option<String>,
    pub published: Option<bool>,
    pub front_page: Option<bool>,
    pub locked_for_user: Option<bool>,
    pub lock_info: Option<serde_json::Value>,
    pub lock_explanation: Option<String>,
}
