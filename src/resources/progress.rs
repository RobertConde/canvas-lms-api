use crate::{error::Result, http::Requester};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A Canvas background job progress object.
///
/// Poll [`Progress::query()`] until `workflow_state` is `"completed"` or `"failed"`.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct Progress {
    pub id: u64,
    pub context_id: Option<u64>,
    pub context_type: Option<String>,
    pub user_id: Option<u64>,
    pub tag: Option<String>,
    pub completion: Option<f64>,
    pub workflow_state: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub message: Option<String>,
    pub results: Option<serde_json::Value>,
    pub url: Option<String>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Progress {
    /// Fetch the latest state of this background job.
    ///
    /// # Canvas API
    /// `GET /api/v1/progress/:id`
    pub async fn query(&self) -> Result<Progress> {
        let mut p: Progress = self.req().get(&format!("progress/{}", self.id), &[]).await?;
        p.requester = self.requester.clone();
        Ok(p)
    }
}
