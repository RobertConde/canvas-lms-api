use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A day summary in the gradebook history.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Day {
    pub date: Option<String>,
    pub graders: Option<u64>,
}

/// A grader entry in the gradebook history.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Grader {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub assignments: Option<Vec<serde_json::Value>>,
}

/// A specific version of a submission from the gradebook history.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubmissionVersion {
    pub id: Option<u64>,
    pub assignment_id: Option<u64>,
    pub assignment_name: Option<String>,
    pub body: Option<String>,
    pub course_id: Option<u64>,
    pub grade: Option<String>,
    pub grader: Option<String>,
    pub grader_id: Option<u64>,
    pub new_grade: Option<String>,
    pub new_graded_at: Option<DateTime<Utc>>,
    pub new_grader: Option<String>,
    pub previous_grade: Option<String>,
    pub previous_graded_at: Option<DateTime<Utc>>,
    pub previous_grader: Option<String>,
    pub score: Option<f64>,
    pub user_name: Option<String>,
    pub submission_type: Option<String>,
    pub url: Option<String>,
    pub user_id: Option<u64>,
    pub workflow_state: Option<String>,
}

/// The version history for a single submission.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubmissionHistory {
    pub submission_id: Option<u64>,
    pub versions: Option<Vec<SubmissionVersion>>,
}
