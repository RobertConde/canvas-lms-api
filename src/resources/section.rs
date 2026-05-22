use crate::{error::Result, http::Requester, pagination::PageStream, params::wrap_params};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::resources::{enrollment::Enrollment, progress::Progress, submission::Submission};

/// Parameters for updating a Canvas section.
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateSectionParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restrict_enrollments_to_section_dates: Option<bool>,
}

/// Parameters for enrolling a user in a section.
#[derive(Debug, Default, Clone, Serialize)]
pub struct EnrollUserParams {
    pub user_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrollment_state: Option<String>,
}

/// A section within a Canvas course.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
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

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Section {
    /// Update this section's name or dates.
    ///
    /// # Canvas API
    /// `PUT /api/v1/sections/:id`
    pub async fn edit(&self, params: UpdateSectionParams) -> Result<Section> {
        let form = wrap_params("course_section", &params);
        let mut s: Section = self
            .req()
            .put(&format!("sections/{}", self.id), &form)
            .await?;
        s.requester = self.requester.clone();
        Ok(s)
    }

    /// Delete this section.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/sections/:id`
    pub async fn delete(&self) -> Result<Section> {
        let mut s: Section = self
            .req()
            .delete(&format!("sections/{}", self.id), &[])
            .await?;
        s.requester = self.requester.clone();
        Ok(s)
    }

    /// Enroll a user in this section.
    ///
    /// # Canvas API
    /// `POST /api/v1/sections/:section_id/enrollments`
    pub async fn enroll_user(&self, params: EnrollUserParams) -> Result<Enrollment> {
        let form = wrap_params("enrollment", &params);
        let mut e: Enrollment = self
            .req()
            .post(&format!("sections/{}/enrollments", self.id), &form)
            .await?;
        e.requester = self.requester.clone();
        Ok(e)
    }

    /// Stream all enrollments in this section.
    ///
    /// # Canvas API
    /// `GET /api/v1/sections/:section_id/enrollments`
    pub fn get_enrollments(&self) -> PageStream<Enrollment> {
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("sections/{}/enrollments", self.id),
            vec![],
            |mut e: Enrollment, req| {
                e.requester = Some(Arc::clone(&req));
                e
            },
        )
    }

    /// Move this section to another course (cross-list).
    ///
    /// # Canvas API
    /// `POST /api/v1/sections/:id/crosslist/:new_course_id`
    pub async fn cross_list_section(&self, new_course_id: u64) -> Result<Section> {
        let mut s: Section = self
            .req()
            .post(
                &format!("sections/{}/crosslist/{new_course_id}", self.id),
                &[],
            )
            .await?;
        s.requester = self.requester.clone();
        Ok(s)
    }

    /// Undo cross-listing of this section.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/sections/:id/crosslist`
    pub async fn decross_list_section(&self) -> Result<Section> {
        let mut s: Section = self
            .req()
            .delete(&format!("sections/{}/crosslist", self.id), &[])
            .await?;
        s.requester = self.requester.clone();
        Ok(s)
    }

    /// Get the assignment override for a specific assignment in this section.
    ///
    /// # Canvas API
    /// `GET /api/v1/sections/:course_section_id/assignments/:assignment_id/override`
    pub async fn get_assignment_override(&self, assignment_id: u64) -> Result<serde_json::Value> {
        self.req()
            .get(
                &format!("sections/{}/assignments/{assignment_id}/override", self.id),
                &[],
            )
            .await
    }

    /// Stream submissions for multiple assignments in this section.
    ///
    /// # Canvas API
    /// `GET /api/v1/sections/:section_id/students/submissions`
    pub fn get_multiple_submissions(&self) -> PageStream<Submission> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("sections/{}/students/submissions", self.id),
            vec![],
        )
    }

    /// Bulk-update grades for this section asynchronously.
    ///
    /// # Canvas API
    /// `POST /api/v1/sections/:section_id/submissions/update_grades`
    pub async fn submissions_bulk_update(&self, params: &[(String, String)]) -> Result<Progress> {
        let mut p: Progress = self
            .req()
            .post(
                &format!("sections/{}/submissions/update_grades", self.id),
                params,
            )
            .await?;
        p.requester = self.requester.clone();
        Ok(p)
    }
}
