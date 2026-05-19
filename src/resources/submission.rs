use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A student's submission for a Canvas assignment.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Submission {
    pub id: u64,
    pub assignment_id: Option<u64>,
    pub course_id: Option<u64>,
    pub user_id: Option<u64>,
    pub body: Option<String>,
    pub url: Option<String>,
    pub grade: Option<String>,
    pub score: Option<f64>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub graded_at: Option<DateTime<Utc>>,
    pub grader_id: Option<i64>,
    pub late: Option<bool>,
    pub missing: Option<bool>,
    pub excused: Option<bool>,
    pub attempt: Option<u64>,
    pub workflow_state: Option<String>,
    pub submission_type: Option<String>,
    pub preview_url: Option<String>,
}
