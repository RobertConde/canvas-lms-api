use crate::resources::types::{EnrollmentType, WorkflowState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Enrollment {
    pub id: u64,
    pub course_id: Option<u64>,
    pub course_section_id: Option<u64>,
    pub user_id: Option<u64>,
    pub enrollment_state: Option<WorkflowState>,
    #[serde(rename = "type")]
    pub enrollment_type: Option<EnrollmentType>,
    pub role: Option<String>,
    pub role_id: Option<u64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub html_url: Option<String>,
    pub grades: Option<EnrollmentGrades>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnrollmentGrades {
    pub html_url: Option<String>,
    pub current_score: Option<f64>,
    pub final_score: Option<f64>,
    pub current_grade: Option<String>,
    pub final_grade: Option<String>,
}
