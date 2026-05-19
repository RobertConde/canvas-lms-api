use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Group {
    pub id: u64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub followed_by_user: Option<bool>,
    pub join_level: Option<String>,
    pub members_count: Option<u64>,
    pub avatar_url: Option<String>,
    pub course_id: Option<u64>,
    pub role: Option<String>,
    pub group_category_id: Option<u64>,
    pub sis_group_id: Option<String>,
    pub sis_import_id: Option<u64>,
    pub storage_quota_mb: Option<u64>,
    pub permissions: Option<serde_json::Value>,
}
