use crate::{
    error::{CanvasError, Result},
    http::Requester,
    params::flatten_params,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Parameters for updating a course tab.
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateTabParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
}

/// A navigation tab in a Canvas course or account.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct Tab {
    pub id: Option<String>,
    pub html_url: Option<String>,
    pub full_url: Option<String>,
    pub label: Option<String>,
    #[serde(rename = "type")]
    pub tab_type: Option<String>,
    pub hidden: Option<bool>,
    pub visibility: Option<String>,
    pub position: Option<u64>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
    /// Set when the tab was retrieved from a course. `None` for group tabs.
    #[serde(skip)]
    pub course_id: Option<u64>,
}

impl Tab {
    /// Update this tab's position or visibility.
    ///
    /// Only course tabs can be updated; group tabs raise an error.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/tabs/:tab_id`
    pub async fn update(&self, params: UpdateTabParams) -> Result<Tab> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Can only update tabs from a Course.".to_string(),
            errors: vec![],
        })?;
        let tab_id = self.id.as_deref().unwrap_or("");
        let form = flatten_params(&serde_json::to_value(&params).unwrap());
        let mut tab: Tab = self
            .req()
            .put(&format!("courses/{course_id}/tabs/{tab_id}"), &form)
            .await?;
        tab.requester = self.requester.clone();
        tab.course_id = self.course_id;
        Ok(tab)
    }
}
