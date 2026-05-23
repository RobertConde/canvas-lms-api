use crate::{error::Result, http::Requester, params::wrap_params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// A Canvas external (LTI) tool.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct ExternalTool {
    pub id: u64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub consumer_key: Option<String>,
    pub course_id: Option<u64>,
    pub account_id: Option<u64>,
    pub workflow_state: Option<String>,
    pub privacy_level: Option<String>,
    pub custom_fields: Option<Value>,
    pub course_navigation: Option<Value>,
    pub account_navigation: Option<Value>,
    pub user_navigation: Option<Value>,
    pub editor_button: Option<Value>,
    pub resource_selection: Option<Value>,
    pub homework_submission: Option<Value>,
    pub selection_width: Option<u64>,
    pub selection_height: Option<u64>,
    pub icon_url: Option<String>,
    pub is_rce_favorite: Option<bool>,
    pub is_top_nav_favorite: Option<bool>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

/// Parameters for creating or updating an external tool.
#[derive(Debug, Default, Clone, Serialize)]
pub struct ExternalToolParams {
    pub name: Option<String>,
    pub privacy_level: Option<String>,
    pub consumer_key: Option<String>,
    pub shared_secret: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub icon_url: Option<String>,
    pub text: Option<String>,
    pub is_rce_favorite: Option<bool>,
}

impl ExternalTool {
    fn parent_type(&self) -> &'static str {
        if self.course_id.is_some() {
            "course"
        } else {
            "account"
        }
    }

    fn parent_id(&self) -> u64 {
        self.course_id
            .or(self.account_id)
            .expect("ExternalTool missing course_id and account_id")
    }

    /// Update this external tool.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/external_tools/:id`
    /// `PUT /api/v1/accounts/:account_id/external_tools/:id`
    pub async fn edit(&self, params: ExternalToolParams) -> Result<ExternalTool> {
        let form = wrap_params("external_tool", &params);
        let mut tool: ExternalTool = self
            .req()
            .put(
                &format!(
                    "{}s/{}/external_tools/{}",
                    self.parent_type(),
                    self.parent_id(),
                    self.id
                ),
                &form,
            )
            .await?;
        tool.requester = self.requester.clone();
        Ok(tool)
    }

    /// Delete this external tool.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/external_tools/:id`
    /// `DELETE /api/v1/accounts/:account_id/external_tools/:id`
    pub async fn delete(&self) -> Result<ExternalTool> {
        let mut tool: ExternalTool = self
            .req()
            .delete(
                &format!(
                    "{}s/{}/external_tools/{}",
                    self.parent_type(),
                    self.parent_id(),
                    self.id
                ),
                &[],
            )
            .await?;
        tool.requester = self.requester.clone();
        Ok(tool)
    }

    /// Fetch the parent resource (Course or Account) that owns this tool.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id`
    /// `GET /api/v1/accounts/:account_id`
    pub async fn get_parent(&self) -> Result<Value> {
        self.req()
            .get(
                &format!("{}s/{}", self.parent_type(), self.parent_id()),
                &[],
            )
            .await
    }

    /// Get the sessionless launch URL for this external tool.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/external_tools/sessionless_launch`
    /// `GET /api/v1/accounts/:account_id/external_tools/sessionless_launch`
    pub async fn get_sessionless_launch_url(&self, params: &[(String, String)]) -> Result<Value> {
        self.req()
            .get(
                &format!(
                    "{}s/{}/external_tools/sessionless_launch",
                    self.parent_type(),
                    self.parent_id()
                ),
                params,
            )
            .await
    }
}
