use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A pairing code used to link an observer to a student.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PairingCode {
    pub user_id: Option<u64>,
    pub code: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub workflow_state: Option<String>,
}
