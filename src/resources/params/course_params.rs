use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CreateCourseParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub course_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public_to_auth_users: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_syllabus: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_student_wiki_edits: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_wiki_comments: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_student_forum_attachments: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_enrollment: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_enrollment: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restrict_enrollments_to_course_dates: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sis_course_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_final_grades: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_assignment_group_weights: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateCourseParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub course_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syllabus_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_final_grades: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
}
