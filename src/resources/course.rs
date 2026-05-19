use crate::{
    error::Result,
    http::Requester,
    pagination::PageStream,
    resources::{
        assignment::Assignment,
        enrollment::Enrollment,
        section::Section,
        types::WorkflowState,
        user::User,
    },
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Course {
    pub id: u64,
    pub name: Option<String>,
    pub course_code: Option<String>,
    pub workflow_state: Option<WorkflowState>,
    pub account_id: Option<u64>,
    pub root_account_id: Option<u64>,
    pub enrollment_term_id: Option<u64>,
    pub sis_course_id: Option<String>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub grading_standard_id: Option<u64>,
    pub is_public: Option<bool>,
    pub license: Option<String>,
    pub locale: Option<String>,
    pub time_zone: Option<String>,
    pub total_students: Option<u64>,
    pub default_view: Option<String>,
    pub syllabus_body: Option<String>,
    pub public_description: Option<String>,
    pub hide_final_grades: Option<bool>,
    pub apply_assignment_group_weights: Option<bool>,
    pub restrict_enrollments_to_course_dates: Option<bool>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Course {
    fn req(&self) -> &Arc<Requester> {
        self.requester.as_ref().expect("requester not initialized")
    }

    /// Stream all assignments in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments`
    pub fn get_assignments(&self) -> PageStream<Assignment> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/assignments", self.id),
            vec![],
        )
    }

    /// Fetch a single assignment.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments/:id`
    pub async fn get_assignment(&self, assignment_id: u64) -> Result<Assignment> {
        self.req()
            .get(
                &format!("courses/{}/assignments/{assignment_id}", self.id),
                &[],
            )
            .await
    }

    /// Stream all sections in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/sections`
    pub fn get_sections(&self) -> PageStream<Section> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/sections", self.id),
            vec![],
        )
    }

    /// Stream all enrollments in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/enrollments`
    pub fn get_enrollments(&self) -> PageStream<Enrollment> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/enrollments", self.id),
            vec![],
        )
    }

    /// Stream all users in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/users`
    pub fn get_users(&self) -> PageStream<User> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/users", self.id),
            vec![],
        )
    }
}
