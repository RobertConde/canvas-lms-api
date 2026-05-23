use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A Canvas authentication event log entry.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthenticationEvent {
    pub created_at: Option<DateTime<Utc>>,
    pub event_type: Option<String>,
    pub pseudonym_id: Option<u64>,
    pub user_id: Option<u64>,
}
