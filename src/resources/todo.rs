use crate::{error::Result, http::Requester};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A Canvas to-do item for the current user.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct Todo {
    #[serde(rename = "type")]
    pub todo_type: Option<String>,
    pub assignment: Option<serde_json::Value>,
    pub quiz: Option<serde_json::Value>,
    pub context_type: Option<String>,
    pub course_id: Option<u64>,
    pub group_id: Option<u64>,
    pub html_url: Option<String>,
    pub ignore: Option<String>,
    pub ignore_permanently: Option<String>,
    pub needs_grading_count: Option<u64>,
    pub due_at: Option<DateTime<Utc>>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Todo {
    /// Ignore this to-do item for the current user until it changes.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/users/self/todo/:asset_type/:asset_id`
    pub async fn delete(&self, asset_type: &str, asset_id: u64) -> Result<()> {
        self.req()
            .delete_void(&format!("users/self/todo/{asset_type}/{asset_id}"))
            .await
    }
}
