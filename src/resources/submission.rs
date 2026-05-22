use crate::{
    error::{CanvasError, Result},
    http::Requester,
    pagination::PageStream,
    params::wrap_params,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Parameters for editing a Canvas submission (grading, comments, etc).
#[derive(Debug, Default, Clone, Serialize)]
pub struct EditSubmissionParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub posted_grade: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excuse: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub late_policy_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds_late_override: Option<u64>,
}

/// A student's submission for a Canvas assignment.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
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

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Submission {
    fn course_assignment_user_prefix(&self) -> Result<String> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Submission has no course_id".to_string(),
            errors: vec![],
        })?;
        let assignment_id = self.assignment_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Submission has no assignment_id".to_string(),
            errors: vec![],
        })?;
        let user_id = self.user_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Submission has no user_id".to_string(),
            errors: vec![],
        })?;
        Ok(format!(
            "courses/{course_id}/assignments/{assignment_id}/submissions/{user_id}"
        ))
    }

    fn propagate(&self, s: &mut Submission) {
        s.requester = self.requester.clone();
        s.course_id = self.course_id;
    }

    /// Edit this submission's grade or comment.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/assignments/:assignment_id/submissions/:user_id`
    pub async fn edit(&self, params: EditSubmissionParams) -> Result<Submission> {
        let prefix = self.course_assignment_user_prefix()?;
        let form = wrap_params("submission", &params);
        let mut s: Submission = self.req().put(&prefix, &form).await?;
        self.propagate(&mut s);
        Ok(s)
    }

    /// Mark this submission as read.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/assignments/:assignment_id/submissions/:user_id/read`
    pub async fn mark_read(&self) -> Result<()> {
        let prefix = self.course_assignment_user_prefix()?;
        self.req().put_void(&format!("{prefix}/read")).await
    }

    /// Mark this submission as unread.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/assignments/:assignment_id/submissions/:user_id/read`
    pub async fn mark_unread(&self) -> Result<()> {
        let prefix = self.course_assignment_user_prefix()?;
        self.req().delete_void(&format!("{prefix}/read")).await
    }

    /// Create a peer review for this submission.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/assignments/:assignment_id/submissions/:user_id/peer_reviews`
    pub async fn create_submission_peer_review(
        &self,
        user_id: u64,
    ) -> Result<serde_json::Value> {
        let prefix = self.course_assignment_user_prefix()?;
        let params = vec![("user_id".to_string(), user_id.to_string())];
        self.req()
            .post(&format!("{prefix}/peer_reviews"), &params)
            .await
    }

    /// Delete a peer review for this submission.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/assignments/:assignment_id/submissions/:user_id/peer_reviews`
    pub async fn delete_submission_peer_review(
        &self,
        user_id: u64,
    ) -> Result<serde_json::Value> {
        let prefix = self.course_assignment_user_prefix()?;
        let params = vec![("user_id".to_string(), user_id.to_string())];
        self.req()
            .delete(&format!("{prefix}/peer_reviews"), &params)
            .await
    }

    /// Stream all peer reviews for this submission.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments/:assignment_id/submissions/:user_id/peer_reviews`
    pub fn get_submission_peer_reviews(&self) -> PageStream<serde_json::Value> {
        let prefix = self
            .course_assignment_user_prefix()
            .unwrap_or_default();
        PageStream::new(
            Arc::clone(self.req()),
            &format!("{prefix}/peer_reviews"),
            vec![],
        )
    }
}
