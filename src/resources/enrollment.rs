use crate::{
    error::{CanvasError, Result},
    http::Requester,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use crate::resources::types::{EnrollmentType, WorkflowState};

/// A Canvas enrollment — the relationship between a user and a course section.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
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

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

/// Grade summary for an enrollment.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnrollmentGrades {
    pub html_url: Option<String>,
    pub current_score: Option<f64>,
    pub final_score: Option<f64>,
    pub current_grade: Option<String>,
    pub final_grade: Option<String>,
}

impl Enrollment {
    fn course_id(&self) -> u64 {
        self.course_id.unwrap_or_default()
    }

    /// Accept a pending course invitation.
    ///
    /// Returns `true` if the invitation was accepted.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/enrollments/:id/accept`
    pub async fn accept(&self) -> Result<bool> {
        let val: Value = self
            .req()
            .post(
                &format!("courses/{}/enrollments/{}/accept", self.course_id(), self.id),
                &[],
            )
            .await?;
        Ok(val.get("success").and_then(Value::as_bool).unwrap_or(false))
    }

    /// Reject a pending course invitation.
    ///
    /// Returns `true` if the invitation was rejected.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/enrollments/:id/reject`
    pub async fn reject(&self) -> Result<bool> {
        let val: Value = self
            .req()
            .post(
                &format!("courses/{}/enrollments/{}/reject", self.course_id(), self.id),
                &[],
            )
            .await?;
        Ok(val.get("success").and_then(Value::as_bool).unwrap_or(false))
    }

    /// Delete, conclude, or deactivate this enrollment.
    ///
    /// `task` must be one of: `"conclude"`, `"delete"`, `"inactivate"`, `"deactivate"`.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/enrollments/:id?task=<task>`
    pub async fn deactivate(&self, task: &str) -> Result<Enrollment> {
        const VALID: &[&str] = &["conclude", "delete", "inactivate", "deactivate"];
        if !VALID.contains(&task) {
            return Err(CanvasError::BadRequest {
                message: format!(
                    "{task} is not a valid task. Use one of: {}",
                    VALID.join(", ")
                ),
                errors: vec![],
            });
        }
        let params = vec![("task".to_string(), task.to_string())];
        let mut e: Enrollment = self
            .req()
            .delete(
                &format!("courses/{}/enrollments/{}", self.course_id(), self.id),
                &params,
            )
            .await?;
        e.requester = self.requester.clone();
        Ok(e)
    }

    /// Re-activate an inactive enrollment.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/enrollments/:id/reactivate`
    pub async fn reactivate(&self) -> Result<Enrollment> {
        let mut e: Enrollment = self
            .req()
            .put(
                &format!(
                    "courses/{}/enrollments/{}/reactivate",
                    self.course_id(),
                    self.id
                ),
                &[],
            )
            .await?;
        e.requester = self.requester.clone();
        Ok(e)
    }
}
