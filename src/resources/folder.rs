use crate::{error::Result, http::Requester, pagination::PageStream, params::flatten_params};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::resources::file::File;

/// Parameters for updating a Canvas folder.
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateFolderParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
}

/// A folder in the Canvas file storage system.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct Folder {
    pub id: u64,
    pub name: Option<String>,
    pub full_name: Option<String>,
    pub context_id: Option<u64>,
    pub context_type: Option<String>,
    pub parent_folder_id: Option<u64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub lock_at: Option<DateTime<Utc>>,
    pub unlock_at: Option<DateTime<Utc>>,
    pub position: Option<u64>,
    pub locked: Option<bool>,
    pub folders_url: Option<String>,
    pub files_url: Option<String>,
    pub files_count: Option<u64>,
    pub folders_count: Option<u64>,
    pub hidden: Option<bool>,
    pub locked_for_user: Option<bool>,
    pub hidden_for_user: Option<bool>,
    pub for_submissions: Option<bool>,
    pub can_upload: Option<bool>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Folder {
    /// Update this folder's name, parent, or lock state.
    ///
    /// # Canvas API
    /// `PUT /api/v1/folders/:id`
    pub async fn update(&self, params: UpdateFolderParams) -> Result<Folder> {
        let form = flatten_params(&serde_json::to_value(&params).unwrap());
        let mut f: Folder = self
            .req()
            .put(&format!("folders/{}", self.id), &form)
            .await?;
        f.requester = self.requester.clone();
        Ok(f)
    }

    /// Delete this folder.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/folders/:id`
    pub async fn delete(&self) -> Result<Folder> {
        let mut f: Folder = self
            .req()
            .delete(&format!("folders/{}", self.id), &[])
            .await?;
        f.requester = self.requester.clone();
        Ok(f)
    }

    /// Stream all files in this folder.
    ///
    /// # Canvas API
    /// `GET /api/v1/folders/:id/files`
    pub fn get_files(&self) -> PageStream<File> {
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("folders/{}/files", self.id),
            vec![],
            |mut f: File, req| {
                f.requester = Some(Arc::clone(&req));
                f
            },
        )
    }

    /// Stream all subfolders in this folder.
    ///
    /// # Canvas API
    /// `GET /api/v1/folders/:id/folders`
    pub fn get_folders(&self) -> PageStream<Folder> {
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("folders/{}/folders", self.id),
            vec![],
            |mut f: Folder, req| {
                f.requester = Some(Arc::clone(&req));
                f
            },
        )
    }

    /// Create a subfolder inside this folder.
    ///
    /// # Canvas API
    /// `POST /api/v1/folders/:id/folders`
    pub async fn create_folder(&self, name: &str) -> Result<Folder> {
        let params = vec![("name".to_string(), name.to_string())];
        let mut f: Folder = self
            .req()
            .post(&format!("folders/{}/folders", self.id), &params)
            .await?;
        f.requester = self.requester.clone();
        Ok(f)
    }

    /// Copy a file into this folder.
    ///
    /// # Canvas API
    /// `POST /api/v1/folders/:dest_folder_id/copy_file`
    pub async fn copy_file(&self, source_file_id: u64) -> Result<File> {
        let params = vec![(
            "source_file_id".to_string(),
            source_file_id.to_string(),
        )];
        let mut f: File = self
            .req()
            .post(&format!("folders/{}/copy_file", self.id), &params)
            .await?;
        f.requester = self.requester.clone();
        Ok(f)
    }
}
