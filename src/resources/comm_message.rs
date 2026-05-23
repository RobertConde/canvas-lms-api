use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A Canvas communication message (email, SMS, push notification, etc.).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommMessage {
    pub id: Option<u64>,
    pub created_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub workflow_state: Option<String>,
    pub from: Option<String>,
    pub from_name: Option<String>,
    pub to: Option<String>,
    pub reply_to: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub html_body: Option<String>,
    pub author_id: Option<u64>,
}
