use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CreateQuizParams {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quiz_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shuffle_answers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_attempts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlock_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,
}
