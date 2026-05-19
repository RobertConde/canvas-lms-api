use serde::{Deserialize, Serialize};

/// A single grade change audit event.
///
/// Returned by `GET /api/v1/audit/grade_change/courses/:course_id` (and
/// the equivalent grader/student/assignment variants). The response body
/// wraps events in an `{ "events": [...] }` object; [`PageStream`] handles
/// this automatically via its object-branch extraction.
///
/// [`PageStream`]: crate::PageStream
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GradeChangeEvent {
    pub id: Option<String>,
    pub created_at: Option<String>,
    pub event_type: Option<String>,
    pub grade_before: Option<String>,
    pub grade_after: Option<String>,
    pub grade_current: Option<String>,
    pub excused_before: Option<bool>,
    pub excused_after: Option<bool>,
    pub point_possible_before: Option<f64>,
    pub point_possible_after: Option<f64>,
    pub graded_anonymously: Option<bool>,
    pub course_id: Option<u64>,
    pub assignment_id: Option<u64>,
    pub student_id: Option<u64>,
    pub grader_id: Option<u64>,
    pub submission_id: Option<u64>,
}
