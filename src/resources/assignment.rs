use crate::resources::types::{SubmissionType, WorkflowState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A Canvas assignment.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Assignment {
    pub id: u64,
    pub course_id: Option<u64>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub unlock_at: Option<DateTime<Utc>>,
    pub lock_at: Option<DateTime<Utc>>,
    pub points_possible: Option<f64>,
    pub grading_type: Option<String>,
    pub assignment_group_id: Option<u64>,
    pub workflow_state: Option<WorkflowState>,
    pub submission_types: Option<Vec<SubmissionType>>,
    pub published: Option<bool>,
    pub muted: Option<bool>,
    pub html_url: Option<String>,
    pub has_overrides: Option<bool>,
    pub needs_grading_count: Option<u64>,
    pub position: Option<u64>,
    pub omit_from_final_grade: Option<bool>,
    pub locked_for_user: Option<bool>,
}
