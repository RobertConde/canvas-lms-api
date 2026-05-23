use crate::{
    error::{CanvasError, Result},
    http::Requester,
    pagination::PageStream,
    params::wrap_params,
    resources::{progress::Progress, submission::Submission, user::User},
    upload::{initiate_and_upload, UploadRequest},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::resources::types::{SubmissionType, WorkflowState};

/// Parameters for creating or editing a Canvas assignment.
#[derive(Debug, Default, Clone, Serialize)]
pub struct AssignmentParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submission_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_extensions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turnitin_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_attempts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points_possible: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grading_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlock_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub omit_from_final_grade: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignment_group_id: Option<u64>,
}

/// Parameters for submitting an assignment.
#[derive(Debug, Default, Clone, Serialize)]
pub struct SubmitAssignmentParams {
    pub submission_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_comment_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_comment_type: Option<String>,
}

/// Parameters for creating an assignment override.
#[derive(Debug, Default, Clone, Serialize)]
pub struct AssignmentOverrideParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub student_ids: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub course_section_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlock_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_at: Option<DateTime<Utc>>,
}

/// Parameters for creating or updating an assignment group.
#[derive(Debug, Default, Clone, Serialize)]
pub struct AssignmentGroupParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
}

/// A Canvas assignment.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
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

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Assignment {
    fn course_prefix(&self) -> Result<String> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Assignment has no course_id".to_string(),
            errors: vec![],
        })?;
        Ok(format!("courses/{course_id}/assignments/{}", self.id))
    }

    fn propagate(&self, a: &mut Assignment) {
        a.requester = self.requester.clone();
        a.course_id = self.course_id;
    }

    fn propagate_sub(&self, s: &mut Submission) {
        s.requester = self.requester.clone();
        s.course_id = self.course_id;
    }

    fn propagate_override(&self, o: &mut AssignmentOverride) {
        o.requester = self.requester.clone();
        o.course_id = self.course_id;
    }

    /// Edit this assignment.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/assignments/:id`
    pub async fn edit(&self, params: AssignmentParams) -> Result<Assignment> {
        let prefix = self.course_prefix()?;
        let form = wrap_params("assignment", &params);
        let mut a: Assignment = self.req().put(&prefix, &form).await?;
        self.propagate(&mut a);
        Ok(a)
    }

    /// Delete this assignment.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/assignments/:id`
    pub async fn delete(&self) -> Result<Assignment> {
        let prefix = self.course_prefix()?;
        let mut a: Assignment = self.req().delete(&prefix, &[]).await?;
        self.propagate(&mut a);
        Ok(a)
    }

    /// Stream all submissions for this assignment.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments/:id/submissions`
    pub fn get_submissions(&self) -> PageStream<Submission> {
        let course_id = self.course_id.unwrap_or(0);
        let assignment_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/assignments/{assignment_id}/submissions"),
            vec![],
            move |mut s: Submission, req| {
                s.requester = Some(Arc::clone(&req));
                s.course_id = Some(course_id);
                s
            },
        )
    }

    /// Fetch a single submission by user ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments/:id/submissions/:user_id`
    pub async fn get_submission(&self, user_id: u64) -> Result<Submission> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Assignment has no course_id".to_string(),
            errors: vec![],
        })?;
        let mut s: Submission = self
            .req()
            .get(
                &format!(
                    "courses/{course_id}/assignments/{}/submissions/{user_id}",
                    self.id
                ),
                &[],
            )
            .await?;
        self.propagate_sub(&mut s);
        Ok(s)
    }

    /// Submit this assignment.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/assignments/:id/submissions`
    pub async fn submit(&self, params: SubmitAssignmentParams) -> Result<Submission> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Assignment has no course_id".to_string(),
            errors: vec![],
        })?;
        let form = wrap_params("submission", &params);
        let mut s: Submission = self
            .req()
            .post(
                &format!("courses/{course_id}/assignments/{}/submissions", self.id),
                &form,
            )
            .await?;
        self.propagate_sub(&mut s);
        Ok(s)
    }

    /// Stream all overrides for this assignment.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments/:id/overrides`
    pub fn get_overrides(&self) -> PageStream<AssignmentOverride> {
        let course_id = self.course_id.unwrap_or(0);
        let assignment_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/assignments/{assignment_id}/overrides"),
            vec![],
            move |mut o: AssignmentOverride, req| {
                o.requester = Some(Arc::clone(&req));
                o.course_id = Some(course_id);
                o
            },
        )
    }

    /// Fetch a single override by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments/:id/overrides/:override_id`
    pub async fn get_override(&self, override_id: u64) -> Result<AssignmentOverride> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Assignment has no course_id".to_string(),
            errors: vec![],
        })?;
        let mut o: AssignmentOverride = self
            .req()
            .get(
                &format!(
                    "courses/{course_id}/assignments/{}/overrides/{override_id}",
                    self.id
                ),
                &[],
            )
            .await?;
        self.propagate_override(&mut o);
        Ok(o)
    }

    /// Create an override for this assignment.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/assignments/:id/overrides`
    pub async fn create_override(
        &self,
        params: AssignmentOverrideParams,
    ) -> Result<AssignmentOverride> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Assignment has no course_id".to_string(),
            errors: vec![],
        })?;
        let form = wrap_params("assignment_override", &params);
        let mut o: AssignmentOverride = self
            .req()
            .post(
                &format!("courses/{course_id}/assignments/{}/overrides", self.id),
                &form,
            )
            .await?;
        self.propagate_override(&mut o);
        Ok(o)
    }

    /// Stream all peer reviews for this assignment.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments/:id/peer_reviews`
    pub fn get_peer_reviews(&self) -> PageStream<serde_json::Value> {
        let course_id = self.course_id.unwrap_or(0);
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/assignments/{}/peer_reviews", self.id),
            vec![],
        )
    }

    /// Stream all gradeable students for this assignment.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments/:id/gradeable_students`
    pub fn get_gradeable_students(&self) -> PageStream<User> {
        let course_id = self.course_id.unwrap_or(0);
        PageStream::new(
            Arc::clone(self.req()),
            &format!(
                "courses/{course_id}/assignments/{}/gradeable_students",
                self.id
            ),
            vec![],
        )
    }

    /// Set extensions for this assignment for one or more students.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/assignments/:id/extensions`
    pub async fn set_extensions(&self, params: &[(String, String)]) -> Result<serde_json::Value> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Assignment has no course_id".to_string(),
            errors: vec![],
        })?;
        self.req()
            .post(
                &format!("courses/{course_id}/assignments/{}/extensions", self.id),
                params,
            )
            .await
    }

    /// Stream grade change events for this assignment.
    ///
    /// # Canvas API
    /// `GET /api/v1/audit/grade_change/assignments/:id`
    pub fn get_grade_change_events(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("audit/grade_change/assignments/{}", self.id),
            vec![],
        )
    }

    /// Stream students selected for moderation on this assignment.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments/:id/moderated_students`
    pub fn get_students_selected_for_moderation(&self) -> PageStream<User> {
        let course_id = self.course_id.unwrap_or(0);
        PageStream::new(
            Arc::clone(self.req()),
            &format!(
                "courses/{course_id}/assignments/{}/moderated_students",
                self.id
            ),
            vec![],
        )
    }

    /// Select students for moderation on this assignment.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/assignments/:id/moderated_students`
    pub async fn select_students_for_moderation(
        &self,
        student_ids: &[u64],
    ) -> Result<Vec<serde_json::Value>> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Assignment has no course_id".to_string(),
            errors: vec![],
        })?;
        let params: Vec<(String, String)> = student_ids
            .iter()
            .map(|id| ("student_ids[]".to_string(), id.to_string()))
            .collect();
        self.req()
            .post(
                &format!(
                    "courses/{course_id}/assignments/{}/moderated_students",
                    self.id
                ),
                &params,
            )
            .await
    }

    /// Check whether a student's submission needs a provisional grade.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments/:id/provisional_grades/status`
    pub async fn get_provisional_grades_status(
        &self,
        student_id: u64,
    ) -> Result<serde_json::Value> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Assignment has no course_id".to_string(),
            errors: vec![],
        })?;
        let params = vec![("student_id".to_string(), student_id.to_string())];
        self.req()
            .get(
                &format!(
                    "courses/{course_id}/assignments/{}/provisional_grades/status",
                    self.id
                ),
                &params,
            )
            .await
    }

    /// Select which provisional grade the student should receive.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/assignments/:id/provisional_grades/:provisional_grade_id/select`
    pub async fn selected_provisional_grade(
        &self,
        provisional_grade_id: u64,
    ) -> Result<serde_json::Value> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Assignment has no course_id".to_string(),
            errors: vec![],
        })?;
        self.req()
            .put(
                &format!(
                    "courses/{course_id}/assignments/{}/provisional_grades/{provisional_grade_id}/select",
                    self.id
                ),
                &[],
            )
            .await
    }

    /// Publish provisional grades for all submissions on this assignment.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/assignments/:id/provisional_grades/publish`
    pub async fn publish_provisional_grades(&self) -> Result<serde_json::Value> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Assignment has no course_id".to_string(),
            errors: vec![],
        })?;
        self.req()
            .post(
                &format!(
                    "courses/{course_id}/assignments/{}/provisional_grades/publish",
                    self.id
                ),
                &[],
            )
            .await
    }

    /// Show whether a student's submission needs a provisional grade (anonymous grading).
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments/:id/anonymous_provisional_grades/status`
    pub async fn show_provisional_grades_for_student(
        &self,
        anonymous_id: &str,
    ) -> Result<serde_json::Value> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Assignment has no course_id".to_string(),
            errors: vec![],
        })?;
        self.req()
            .get(
                &format!(
                    "courses/{course_id}/assignments/{}/anonymous_provisional_grades/status",
                    self.id
                ),
                &[("anonymous_id".to_string(), anonymous_id.to_string())],
            )
            .await
    }

    /// Bulk-update grades for this assignment asynchronously.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/assignments/:id/submissions/update_grades`
    pub async fn submissions_bulk_update(&self, params: &[(String, String)]) -> Result<Progress> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Assignment has no course_id".to_string(),
            errors: vec![],
        })?;
        let mut p: Progress = self
            .req()
            .post(
                &format!(
                    "courses/{course_id}/assignments/{}/submissions/update_grades",
                    self.id
                ),
                params,
            )
            .await?;
        p.requester = self.requester.clone();
        Ok(p)
    }

    /// Upload a file to a submission for this assignment (two-step Canvas upload).
    ///
    /// `user_id` is the submitting user's Canvas id; pass `0` to use `"self"` is not
    /// supported here — provide the numeric id.
    ///
    /// # Canvas API
    /// Step 1: `POST /api/v1/courses/:c/assignments/:a/submissions/:u/files`
    /// Step 2: POST multipart to the returned upload URL
    pub async fn upload_to_submission(
        &self,
        user_id: u64,
        request: UploadRequest,
        data: Vec<u8>,
    ) -> Result<crate::resources::file::File> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Assignment has no course_id".to_string(),
            errors: vec![],
        })?;
        let endpoint =
            format!("courses/{course_id}/assignments/{}/submissions/{user_id}/files", self.id);
        initiate_and_upload(self.req(), &endpoint, request, data).await
    }
}

/// A Canvas assignment group.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct AssignmentGroup {
    pub id: u64,
    pub name: Option<String>,
    pub group_weight: Option<f64>,
    pub position: Option<u64>,
    pub rules: Option<serde_json::Value>,
    pub assignments: Option<Vec<serde_json::Value>>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
    #[serde(skip)]
    pub course_id: Option<u64>,
}

impl AssignmentGroup {
    /// Edit this assignment group.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/assignment_groups/:id`
    pub async fn edit(&self, params: AssignmentGroupParams) -> Result<AssignmentGroup> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "AssignmentGroup has no course_id".to_string(),
            errors: vec![],
        })?;
        let form = wrap_params("assignment_group", &params);
        let mut g: AssignmentGroup = self
            .req()
            .put(
                &format!("courses/{course_id}/assignment_groups/{}", self.id),
                &form,
            )
            .await?;
        g.requester = self.requester.clone();
        g.course_id = self.course_id;
        Ok(g)
    }

    /// Delete this assignment group.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/assignment_groups/:id`
    pub async fn delete(&self) -> Result<AssignmentGroup> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "AssignmentGroup has no course_id".to_string(),
            errors: vec![],
        })?;
        let mut g: AssignmentGroup = self
            .req()
            .delete(
                &format!("courses/{course_id}/assignment_groups/{}", self.id),
                &[],
            )
            .await?;
        g.requester = self.requester.clone();
        g.course_id = self.course_id;
        Ok(g)
    }
}

/// An override for a Canvas assignment (adjusts due dates for specific students/groups/sections).
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct AssignmentOverride {
    pub id: u64,
    pub assignment_id: Option<u64>,
    pub student_ids: Option<Vec<u64>>,
    pub group_id: Option<u64>,
    pub course_section_id: Option<u64>,
    pub title: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub unlock_at: Option<DateTime<Utc>>,
    pub lock_at: Option<DateTime<Utc>>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
    #[serde(skip)]
    pub course_id: Option<u64>,
}

impl AssignmentOverride {
    fn prefix(&self) -> Result<String> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "AssignmentOverride has no course_id".to_string(),
            errors: vec![],
        })?;
        let assignment_id = self.assignment_id.ok_or_else(|| CanvasError::BadRequest {
            message: "AssignmentOverride has no assignment_id".to_string(),
            errors: vec![],
        })?;
        Ok(format!(
            "courses/{course_id}/assignments/{assignment_id}/overrides/{}",
            self.id
        ))
    }

    fn propagate(&self, o: &mut AssignmentOverride) {
        o.requester = self.requester.clone();
        o.course_id = self.course_id;
    }

    /// Edit this override.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/assignments/:assignment_id/overrides/:id`
    pub async fn edit(&self, params: AssignmentOverrideParams) -> Result<AssignmentOverride> {
        let prefix = self.prefix()?;
        let form = wrap_params("assignment_override", &params);
        let mut o: AssignmentOverride = self.req().put(&prefix, &form).await?;
        self.propagate(&mut o);
        Ok(o)
    }

    /// Delete this override.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/assignments/:assignment_id/overrides/:id`
    pub async fn delete(&self) -> Result<AssignmentOverride> {
        let prefix = self.prefix()?;
        let mut o: AssignmentOverride = self.req().delete(&prefix, &[]).await?;
        self.propagate(&mut o);
        Ok(o)
    }
}
