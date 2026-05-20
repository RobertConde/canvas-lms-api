use crate::{error::Result, http::Requester, params::wrap_params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// A Canvas rubric.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct Rubric {
    pub id: u64,
    pub title: Option<String>,
    pub context_id: Option<u64>,
    pub context_type: Option<String>,
    pub course_id: Option<u64>,
    pub account_id: Option<u64>,
    pub points_possible: Option<f64>,
    pub reusable: Option<bool>,
    pub read_only: Option<bool>,
    pub free_form_criterion_comments: Option<bool>,
    pub hide_score_total: Option<bool>,
    pub data: Option<Vec<Value>>,
    pub assessments: Option<Vec<Value>>,
    pub associations: Option<Vec<Value>>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

/// Parameters for creating or updating a rubric.
#[derive(Debug, Default, Clone, Serialize)]
pub struct RubricParams {
    pub title: Option<String>,
    pub free_form_criterion_comments: Option<bool>,
}

impl Rubric {
    fn course_id(&self) -> u64 {
        self.course_id.expect("Rubric missing course_id")
    }

    /// Delete this rubric.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/rubrics/:id`
    pub async fn delete(&self) -> Result<Rubric> {
        let mut rubric: Rubric = self
            .req()
            .delete(
                &format!("courses/{}/rubrics/{}", self.course_id(), self.id),
                &[],
            )
            .await?;
        rubric.requester = self.requester.clone();
        Ok(rubric)
    }

    /// Update this rubric.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/rubrics/:id`
    pub async fn update(&self, params: RubricParams) -> Result<Rubric> {
        let form = wrap_params("rubric", &params);
        let mut rubric: Rubric = self
            .req()
            .put(
                &format!("courses/{}/rubrics/{}", self.course_id(), self.id),
                &form,
            )
            .await?;
        rubric.requester = self.requester.clone();
        Ok(rubric)
    }
}

/// A rubric assessment (a filled-out rubric for a specific submission).
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct RubricAssessment {
    pub id: u64,
    pub rubric_id: Option<u64>,
    pub rubric_association_id: Option<u64>,
    pub score: Option<f64>,
    pub artifact_type: Option<String>,
    pub artifact_id: Option<u64>,
    pub artifact_attempt: Option<u64>,
    pub assessment_type: Option<String>,
    pub assessor_id: Option<u64>,
    pub course_id: Option<u64>,
    pub data: Option<Vec<Value>>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl RubricAssessment {
    fn course_id(&self) -> u64 {
        self.course_id.expect("RubricAssessment missing course_id")
    }

    fn rubric_association_id(&self) -> u64 {
        self.rubric_association_id
            .expect("RubricAssessment missing rubric_association_id")
    }

    /// Delete this rubric assessment.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/rubric_associations/:rubric_association_id/rubric_assessments/:id`
    pub async fn delete(&self) -> Result<RubricAssessment> {
        let mut assessment: RubricAssessment = self
            .req()
            .delete(
                &format!(
                    "courses/{}/rubric_associations/{}/rubric_assessments/{}",
                    self.course_id(),
                    self.rubric_association_id(),
                    self.id
                ),
                &[],
            )
            .await?;
        assessment.requester = self.requester.clone();
        Ok(assessment)
    }

    /// Update this rubric assessment.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/rubric_associations/:rubric_association_id/rubric_assessments/:id`
    pub async fn update(&self, params: &[(String, String)]) -> Result<RubricAssessment> {
        let mut assessment: RubricAssessment = self
            .req()
            .put(
                &format!(
                    "courses/{}/rubric_associations/{}/rubric_assessments/{}",
                    self.course_id(),
                    self.rubric_association_id(),
                    self.id
                ),
                params,
            )
            .await?;
        assessment.requester = self.requester.clone();
        Ok(assessment)
    }
}

/// An association between a rubric and a course assignment or course.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct RubricAssociation {
    pub id: u64,
    pub rubric_id: Option<u64>,
    pub association_id: Option<u64>,
    pub association_type: Option<String>,
    pub use_for_grading: Option<bool>,
    pub course_id: Option<u64>,
    pub summary_data: Option<String>,
    pub purpose: Option<String>,
    pub hide_score_total: Option<bool>,
    pub hide_points: Option<bool>,
    pub hide_outcome_results: Option<bool>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl RubricAssociation {
    fn course_id(&self) -> u64 {
        self.course_id.expect("RubricAssociation missing course_id")
    }

    /// Create a rubric assessment under this association.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/rubric_associations/:id/rubric_assessments`
    pub async fn create_rubric_assessment(
        &self,
        params: &[(String, String)],
    ) -> Result<RubricAssessment> {
        let mut assessment: RubricAssessment = self
            .req()
            .post(
                &format!(
                    "courses/{}/rubric_associations/{}/rubric_assessments",
                    self.course_id(),
                    self.id
                ),
                params,
            )
            .await?;
        assessment.requester = self.requester.clone();
        Ok(assessment)
    }

    /// Delete this rubric association.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/rubric_associations/:id`
    pub async fn delete(&self) -> Result<RubricAssociation> {
        let mut assoc: RubricAssociation = self
            .req()
            .delete(
                &format!(
                    "courses/{}/rubric_associations/{}",
                    self.course_id(),
                    self.id
                ),
                &[],
            )
            .await?;
        assoc.requester = self.requester.clone();
        Ok(assoc)
    }

    /// Update this rubric association.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/rubric_associations/:id`
    pub async fn update(&self, params: &[(String, String)]) -> Result<RubricAssociation> {
        let mut assoc: RubricAssociation = self
            .req()
            .put(
                &format!(
                    "courses/{}/rubric_associations/{}",
                    self.course_id(),
                    self.id
                ),
                params,
            )
            .await?;
        assoc.requester = self.requester.clone();
        Ok(assoc)
    }
}
