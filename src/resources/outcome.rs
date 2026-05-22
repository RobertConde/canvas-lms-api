use crate::{error::Result, http::Requester, pagination::PageStream};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// A Canvas learning outcome.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct Outcome {
    pub id: u64,
    pub url: Option<String>,
    pub context_id: Option<u64>,
    pub context_type: Option<String>,
    pub title: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub vendor_guid: Option<String>,
    pub points_possible: Option<f64>,
    pub mastery_points: Option<f64>,
    pub calculation_method: Option<String>,
    pub calculation_int: Option<u64>,
    pub ratings: Option<Vec<Value>>,
    pub can_edit: Option<bool>,
    pub can_unlink: Option<bool>,
    pub assessed: Option<bool>,
    pub has_updateable_rubrics: Option<bool>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

/// Parameters for updating an outcome.
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateOutcomeParams {
    pub title: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub vendor_guid: Option<String>,
    pub mastery_points: Option<f64>,
    pub calculation_method: Option<String>,
    pub calculation_int: Option<u64>,
}

impl Outcome {
    /// Update this outcome.
    ///
    /// # Canvas API
    /// `PUT /api/v1/outcomes/:id`
    pub async fn update(&self, params: UpdateOutcomeParams) -> Result<Outcome> {
        use crate::params::wrap_params;
        let form = wrap_params("outcome", &params);
        let mut outcome: Outcome = self
            .req()
            .put(&format!("outcomes/{}", self.id), &form)
            .await?;
        outcome.requester = self.requester.clone();
        Ok(outcome)
    }
}

/// A group of learning outcomes.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct OutcomeGroup {
    pub id: u64,
    pub url: Option<String>,
    pub parent_outcome_group: Option<Box<OutcomeGroup>>,
    pub context_id: Option<u64>,
    pub context_type: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub vendor_guid: Option<String>,
    pub subgroups_url: Option<String>,
    pub outcomes_url: Option<String>,
    pub can_edit: Option<bool>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

/// Parameters for updating an outcome group.
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateOutcomeGroupParams {
    pub title: Option<String>,
    pub description: Option<String>,
    pub vendor_guid: Option<String>,
    pub parent_outcome_group_id: Option<u64>,
}

impl OutcomeGroup {
    fn context_path(&self) -> String {
        match self.context_type.as_deref() {
            Some("Course") => format!("courses/{}", self.context_id.unwrap_or_default()),
            Some("Account") => format!("accounts/{}", self.context_id.unwrap_or_default()),
            _ => "global".to_string(),
        }
    }

    /// Update this outcome group.
    ///
    /// # Canvas API
    /// `PUT /api/v1/accounts/:account_id/outcome_groups/:id`
    /// `PUT /api/v1/courses/:course_id/outcome_groups/:id`
    pub async fn update(&self, params: UpdateOutcomeGroupParams) -> Result<OutcomeGroup> {
        use crate::params::wrap_params;
        let form = wrap_params("outcome_group", &params);
        let mut group: OutcomeGroup = self
            .req()
            .put(
                &format!("{}/outcome_groups/{}", self.context_path(), self.id),
                &form,
            )
            .await?;
        group.requester = self.requester.clone();
        Ok(group)
    }

    /// Delete this outcome group.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/accounts/:account_id/outcome_groups/:id`
    pub async fn delete(&self) -> Result<OutcomeGroup> {
        let mut group: OutcomeGroup = self
            .req()
            .delete(
                &format!("{}/outcome_groups/{}", self.context_path(), self.id),
                &[],
            )
            .await?;
        group.requester = self.requester.clone();
        Ok(group)
    }

    /// Stream sub-groups of this outcome group.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/outcome_groups/:id/subgroups`
    pub fn get_subgroups(&self) -> PageStream<OutcomeGroup> {
        let path = format!(
            "{}/outcome_groups/{}/subgroups",
            self.context_path(),
            self.id
        );
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &path,
            vec![],
            |mut g: OutcomeGroup, req| {
                g.requester = Some(Arc::clone(&req));
                g
            },
        )
    }

    /// Create a sub-group under this outcome group.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:account_id/outcome_groups/:id/subgroups`
    pub async fn create_subgroup(&self, title: &str) -> Result<OutcomeGroup> {
        let params = vec![("title".to_string(), title.to_string())];
        let mut group: OutcomeGroup = self
            .req()
            .post(
                &format!(
                    "{}/outcome_groups/{}/subgroups",
                    self.context_path(),
                    self.id
                ),
                &params,
            )
            .await?;
        group.requester = self.requester.clone();
        Ok(group)
    }

    /// Stream outcomes linked to this group.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/outcome_groups/:id/outcomes`
    pub fn get_linked_outcomes(&self) -> PageStream<OutcomeLink> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!(
                "{}/outcome_groups/{}/outcomes",
                self.context_path(),
                self.id
            ),
            vec![],
        )
    }

    /// Link an existing outcome into this group.
    ///
    /// # Canvas API
    /// `PUT /api/v1/accounts/:account_id/outcome_groups/:id/outcomes/:outcome_id`
    pub async fn link_outcome(&self, outcome_id: u64) -> Result<OutcomeLink> {
        self.req()
            .put(
                &format!(
                    "{}/outcome_groups/{}/outcomes/{outcome_id}",
                    self.context_path(),
                    self.id
                ),
                &[],
            )
            .await
    }

    /// Unlink an outcome from this group.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/accounts/:account_id/outcome_groups/:id/outcomes/:outcome_id`
    pub async fn unlink_outcome(&self, outcome_id: u64) -> Result<OutcomeLink> {
        self.req()
            .delete(
                &format!(
                    "{}/outcome_groups/{}/outcomes/{outcome_id}",
                    self.context_path(),
                    self.id
                ),
                &[],
            )
            .await
    }

    /// Import another outcome group into this group.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:account_id/outcome_groups/:id/import`
    /// `POST /api/v1/courses/:course_id/outcome_groups/:id/import`
    pub async fn import_outcome_group(&self, source_group_id: u64) -> Result<OutcomeGroup> {
        let params = vec![(
            "source_outcome_group_id".to_string(),
            source_group_id.to_string(),
        )];
        let mut group: OutcomeGroup = self
            .req()
            .post(
                &format!("{}/outcome_groups/{}/import", self.context_path(), self.id),
                &params,
            )
            .await?;
        group.requester = self.requester.clone();
        Ok(group)
    }
}

/// An association between an outcome and an outcome group.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutcomeLink {
    pub context_id: Option<u64>,
    pub context_type: Option<String>,
    pub url: Option<String>,
    pub outcome_group: Option<Value>,
    pub outcome: Option<Value>,
    pub assessed: Option<bool>,
    pub can_unlink: Option<bool>,
}
