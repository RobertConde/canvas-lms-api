use crate::{error::Result, http::Requester, params::flatten_params};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Parameters for updating a Canvas file.
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateFileParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_duplicate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
}

/// A file stored in Canvas.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct File {
    pub id: u64,
    pub uuid: Option<String>,
    pub folder_id: Option<u64>,
    pub display_name: Option<String>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub url: Option<String>,
    pub size: Option<u64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub unlock_at: Option<DateTime<Utc>>,
    pub locked: Option<bool>,
    pub hidden: Option<bool>,
    pub lock_at: Option<DateTime<Utc>>,
    pub hidden_for_user: Option<bool>,
    pub thumbnail_url: Option<String>,
    pub modified_at: Option<DateTime<Utc>>,
    pub mime_class: Option<String>,
    pub media_entry_id: Option<String>,
    pub locked_for_user: Option<bool>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl File {
    /// Update metadata (name, folder, lock state) of this file.
    ///
    /// # Canvas API
    /// `PUT /api/v1/files/:id`
    pub async fn update(&self, params: UpdateFileParams) -> Result<File> {
        let form = flatten_params(&serde_json::to_value(&params).unwrap());
        let mut f: File = self.req().put(&format!("files/{}", self.id), &form).await?;
        f.requester = self.requester.clone();
        Ok(f)
    }

    /// Delete this file.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/files/:id`
    pub async fn delete(&self) -> Result<File> {
        let mut f: File = self
            .req()
            .delete(&format!("files/{}", self.id), &[])
            .await?;
        f.requester = self.requester.clone();
        Ok(f)
    }

    /// Download the raw bytes of this file's content.
    ///
    /// Uses the file's `url` field (an absolute Canvas file URL).
    pub async fn get_contents(&self) -> Result<Vec<u8>> {
        let url = self.url.as_deref().unwrap_or("");
        self.req().get_url_bytes(url).await
    }

    /// Download this file to a local path.
    pub async fn download(&self, path: &std::path::Path) -> Result<()> {
        let bytes = self.get_contents().await?;
        std::fs::write(path, bytes)?;
        Ok(())
    }
}
