use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A Canvas quiz.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Quiz {
    pub id: u64,
    pub course_id: Option<u64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub quiz_type: Option<String>,
    pub time_limit: Option<u64>,
    pub shuffle_answers: Option<bool>,
    pub show_correct_answers: Option<bool>,
    pub scoring_policy: Option<String>,
    pub allowed_attempts: Option<i64>,
    pub one_question_at_a_time: Option<bool>,
    pub question_count: Option<u64>,
    pub points_possible: Option<f64>,
    pub due_at: Option<DateTime<Utc>>,
    pub lock_at: Option<DateTime<Utc>>,
    pub unlock_at: Option<DateTime<Utc>>,
    pub published: Option<bool>,
    pub workflow_state: Option<String>,
    pub html_url: Option<String>,
}
