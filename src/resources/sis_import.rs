use crate::{error::Result, http::Requester, resources::progress::Progress};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A SIS import job.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SisImport {
    pub id: u64,
    pub account_id: Option<u64>,
    pub workflow_state: Option<String>,
    pub data: Option<SisImportData>,
    pub progress: Option<f64>,
    pub errors_attachment: Option<serde_json::Value>,
    pub processing_warnings: Option<Vec<Vec<String>>>,
    pub processing_errors: Option<Vec<Vec<String>>>,
    pub batch_mode: Option<bool>,
    pub batch_mode_term_id: Option<String>,
    pub multi_term_batch_mode: Option<bool>,
    pub skip_deletes: Option<bool>,
    pub override_sis_stickiness: Option<bool>,
    pub add_sis_stickiness: Option<bool>,
    pub clear_sis_stickiness: Option<bool>,
    pub diffing_data_set_identifier: Option<String>,
    pub diffed_against_import_id: Option<u64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

/// Import type and count summary for a SIS import.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SisImportData {
    pub import_type: Option<String>,
    pub supplied_batches: Option<Vec<String>>,
    pub counts: Option<SisImportCounts>,
}

/// Record counts produced by a SIS import.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SisImportCounts {
    pub accounts: Option<u64>,
    pub terms: Option<u64>,
    pub abstract_courses: Option<u64>,
    pub courses: Option<u64>,
    pub sections: Option<u64>,
    pub xlists: Option<u64>,
    pub users: Option<u64>,
    pub enrollments: Option<u64>,
    pub groups: Option<u64>,
    pub group_memberships: Option<u64>,
    pub grade_publishing_results: Option<u64>,
    pub batch_courses_deleted: Option<u64>,
    pub batch_sections_deleted: Option<u64>,
    pub batch_enrollments_deleted: Option<u64>,
}

impl SisImport {
    fn req(&self) -> &Arc<Requester> {
        self.requester.as_ref().expect("requester not initialized")
    }

    fn account_id(&self) -> u64 {
        self.account_id.expect("SisImport missing account_id")
    }

    /// Abort this SIS import.
    ///
    /// # Canvas API
    /// `PUT /api/v1/accounts/:account_id/sis_imports/:id/abort`
    pub async fn abort(&self) -> Result<SisImport> {
        let mut import: SisImport = self
            .req()
            .put(
                &format!(
                    "accounts/{}/sis_imports/{}/abort",
                    self.account_id(),
                    self.id
                ),
                &[],
            )
            .await?;
        import.requester = self.requester.clone();
        Ok(import)
    }

    /// Restore workflow states of SIS-imported items.
    ///
    /// # Canvas API
    /// `PUT /api/v1/accounts/:account_id/sis_imports/:id/restore_states`
    pub async fn restore_states(&self) -> Result<Progress> {
        self.req()
            .put(
                &format!(
                    "accounts/{}/sis_imports/{}/restore_states",
                    self.account_id(),
                    self.id
                ),
                &[],
            )
            .await
    }
}
