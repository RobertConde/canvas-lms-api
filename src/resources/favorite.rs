use crate::{error::{CanvasError, Result}, http::Requester};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A Canvas favorite (starred course or group).
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct Favorite {
    pub context_id: Option<u64>,
    pub context_type: Option<String>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Favorite {
    /// Remove this item from the current user's favorites.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/users/self/favorites/:context_type/:context_id`
    pub async fn remove(&self) -> Result<Favorite> {
        let context_type = self.context_type.as_deref().ok_or_else(|| CanvasError::BadRequest {
            message: "Favorite has no context_type".to_string(),
            errors: vec![],
        })?;
        let context_id = self.context_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Favorite has no context_id".to_string(),
            errors: vec![],
        })?;
        let mut f: Favorite = self
            .req()
            .delete(
                &format!("users/self/favorites/{context_type}/{context_id}"),
                &[],
            )
            .await?;
        f.requester = self.requester.clone();
        Ok(f)
    }
}
