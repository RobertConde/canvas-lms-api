use crate::{error::{CanvasError, Result}, http::Requester};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// An external RSS/Atom feed attached to a course or group.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct ExternalFeed {
    pub id: Option<u64>,
    pub display_name: Option<String>,
    pub url: Option<String>,
    pub header_match: Option<String>,
    pub verbosity: Option<String>,
    pub created_at: Option<DateTime<Utc>>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
    #[serde(skip)]
    pub(crate) context: Option<String>,
}

impl ExternalFeed {
    fn prefix(&self) -> Result<String> {
        self.context.clone().ok_or_else(|| CanvasError::BadRequest {
            message: "ExternalFeed has no context".to_string(),
            errors: vec![],
        })
    }

    /// Delete this external feed.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/:context/external_feeds/:id`
    pub async fn delete(&self) -> Result<ExternalFeed> {
        let id = self.id.ok_or_else(|| CanvasError::BadRequest {
            message: "ExternalFeed has no id".to_string(),
            errors: vec![],
        })?;
        let prefix = self.prefix()?;
        let mut f: ExternalFeed = self
            .req()
            .delete(&format!("{prefix}/external_feeds/{id}"), &[])
            .await?;
        f.requester = self.requester.clone();
        f.context = self.context.clone();
        Ok(f)
    }
}
