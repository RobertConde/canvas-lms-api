use crate::{
    error::{CanvasError, Result},
    http::Requester,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A Canvas bookmark saved by the current user.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct Bookmark {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub position: Option<u64>,
    pub data: Option<serde_json::Value>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Bookmark {
    /// Update this bookmark.
    ///
    /// # Canvas API
    /// `PUT /api/v1/users/self/bookmarks/:id`
    pub async fn edit(&self, params: &[(String, String)]) -> Result<Bookmark> {
        let id = self.id.ok_or_else(|| CanvasError::BadRequest {
            message: "Bookmark has no id".to_string(),
            errors: vec![],
        })?;
        let mut b: Bookmark = self
            .req()
            .put(&format!("users/self/bookmarks/{id}"), params)
            .await?;
        b.requester = self.requester.clone();
        Ok(b)
    }

    /// Delete this bookmark.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/users/self/bookmarks/:id`
    pub async fn delete(&self) -> Result<()> {
        let id = self.id.ok_or_else(|| CanvasError::BadRequest {
            message: "Bookmark has no id".to_string(),
            errors: vec![],
        })?;
        self.req()
            .delete_void(&format!("users/self/bookmarks/{id}"))
            .await
    }
}
