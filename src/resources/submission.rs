use crate::{
    error::{CanvasError, Result},
    http::Requester,
    pagination::PageStream,
    params::wrap_params,
    resources::user::User,
    upload::{initiate_and_upload, UploadRequest},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A Canvas peer review assignment.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct PeerReview {
    pub assessor_id: Option<u64>,
    pub asset_id: Option<u64>,
    pub asset_type: Option<String>,
    pub id: Option<u64>,
    pub user_id: Option<u64>,
    pub user: Option<User>,
    pub workflow_state: Option<String>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
    #[serde(skip)]
    pub(crate) course_id: Option<u64>,
    #[serde(skip)]
    pub(crate) assignment_id: Option<u64>,
    #[serde(skip)]
    pub(crate) submission_user_id: Option<u64>,
}

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
    pub async fn create_submission_peer_review(&self, assessor_id: u64) -> Result<PeerReview> {
        let prefix = self.course_assignment_user_prefix()?;
        let params = vec![("user_id".to_string(), assessor_id.to_string())];
        let mut pr: PeerReview = self.req().post(&format!("{prefix}/peer_reviews"), &params).await?;
        pr.requester = self.requester.clone();
        pr.course_id = self.course_id;
        pr.assignment_id = self.assignment_id;
        pr.submission_user_id = self.user_id;
        Ok(pr)
    }

    /// Delete a peer review for this submission.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/assignments/:assignment_id/submissions/:user_id/peer_reviews`
    pub async fn delete_submission_peer_review(&self, assessor_id: u64) -> Result<PeerReview> {
        let prefix = self.course_assignment_user_prefix()?;
        let params = vec![("user_id".to_string(), assessor_id.to_string())];
        let mut pr: PeerReview = self
            .req()
            .delete(&format!("{prefix}/peer_reviews"), &params)
            .await?;
        pr.requester = self.requester.clone();
        pr.course_id = self.course_id;
        pr.assignment_id = self.assignment_id;
        pr.submission_user_id = self.user_id;
        Ok(pr)
    }

    /// Stream all peer reviews for this submission.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments/:assignment_id/submissions/:user_id/peer_reviews`
    pub fn get_submission_peer_reviews(&self) -> PageStream<PeerReview> {
        let prefix = self.course_assignment_user_prefix().unwrap_or_default();
        let course_id = self.course_id;
        let assignment_id = self.assignment_id;
        let submission_user_id = self.user_id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("{prefix}/peer_reviews"),
            vec![],
            move |mut pr: PeerReview, req| {
                pr.requester = Some(Arc::clone(&req));
                pr.course_id = course_id;
                pr.assignment_id = assignment_id;
                pr.submission_user_id = submission_user_id;
                pr
            },
        )
    }

    /// Upload a file as a submission comment attachment.
    ///
    /// Performs the two-step Canvas file upload to
    /// `courses/:course_id/assignments/:assignment_id/submissions/:user_id/comments/files`.
    /// Returns the uploaded [`File`][crate::resources::file::File].
    ///
    /// # Canvas API
    /// Step 1: `POST /api/v1/courses/:c/assignments/:a/submissions/:u/comments/files`
    /// Step 2: POST multipart to the returned upload URL
    pub async fn upload_comment(
        &self,
        request: UploadRequest,
        data: Vec<u8>,
    ) -> Result<crate::resources::file::File> {
        let prefix = self.course_assignment_user_prefix()?;
        let endpoint = format!("{prefix}/comments/files");
        initiate_and_upload(self.req(), &endpoint, request, data).await
    }
}
