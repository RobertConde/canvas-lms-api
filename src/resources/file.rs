use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A file stored in Canvas.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    pub id: u64,
    pub uuid: Option<String>,
    pub folder_id: Option<u64>,
    pub display_name: Option<String>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub url: Option<String>,
    pub size: Option<u64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub unlock_at: Option<DateTime<Utc>>,
    pub locked: Option<bool>,
    pub hidden: Option<bool>,
    pub lock_at: Option<DateTime<Utc>>,
    pub hidden_for_user: Option<bool>,
    pub thumbnail_url: Option<String>,
    pub modified_at: Option<DateTime<Utc>>,
    pub mime_class: Option<String>,
    pub media_entry_id: Option<String>,
    pub locked_for_user: Option<bool>,
}
