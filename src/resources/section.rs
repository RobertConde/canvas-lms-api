use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A section within a Canvas course.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Section {
    pub id: u64,
    pub course_id: Option<u64>,
    pub name: Option<String>,
    pub sis_section_id: Option<String>,
    pub integration_id: Option<String>,
    pub sis_import_id: Option<u64>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub restrict_enrollments_to_section_dates: Option<bool>,
    pub nonxlist_course_id: Option<u64>,
    pub total_students: Option<u64>,
}
