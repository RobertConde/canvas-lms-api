use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A global notification shown to users in a Canvas account.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountNotification {
    pub id: Option<u64>,
    pub subject: Option<String>,
    pub message: Option<String>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub icon: Option<String>,
}
