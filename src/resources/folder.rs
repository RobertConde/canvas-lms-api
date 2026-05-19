use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A folder in the Canvas file storage system.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Folder {
    pub id: u64,
    pub name: Option<String>,
    pub full_name: Option<String>,
    pub context_id: Option<u64>,
    pub context_type: Option<String>,
    pub parent_folder_id: Option<u64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub lock_at: Option<DateTime<Utc>>,
    pub unlock_at: Option<DateTime<Utc>>,
    pub position: Option<u64>,
    pub locked: Option<bool>,
    pub folders_url: Option<String>,
    pub files_url: Option<String>,
    pub files_count: Option<u64>,
    pub folders_count: Option<u64>,
    pub hidden: Option<bool>,
    pub locked_for_user: Option<bool>,
    pub hidden_for_user: Option<bool>,
    pub for_submissions: Option<bool>,
    pub can_upload: Option<bool>,
}
