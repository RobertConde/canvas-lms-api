use crate::{error::Result, http::Requester};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Metadata about a Canvas feature flag option.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Feature {
    pub feature: Option<String>,
    pub display_name: Option<String>,
    pub applies_to: Option<String>,
    pub enable_at: Option<String>,
    pub beta: Option<bool>,
    pub development: Option<bool>,
    pub autoexpand: Option<bool>,
    pub feature_flag: Option<FeatureFlag>,
}

/// The state of a feature flag for a particular context (account, course, or user).
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct FeatureFlag {
    pub feature: Option<String>,
    pub context_type: Option<String>,
    pub context_id: Option<u64>,
    /// One of: `"on"`, `"off"`, `"allowed"`, `"allowed_on"`, `"hidden"`
    pub state: Option<String>,
    pub locked: Option<bool>,
    pub transitions: Option<serde_json::Value>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl FeatureFlag {
    fn flag_path(&self) -> String {
        let ctx = self
            .context_type
            .as_deref()
            .unwrap_or("")
            .to_lowercase()
            + "s";
        let ctx_id = self.context_id.unwrap_or_default();
        let feature = self.feature.as_deref().unwrap_or("");
        format!("{ctx}/{ctx_id}/features/flags/{feature}")
    }

    /// Remove this feature flag, reverting to the inherited value.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/:context_type/:context_id/features/flags/:feature`
    pub async fn delete(&self) -> Result<FeatureFlag> {
        let mut ff: FeatureFlag = self.req().delete(&self.flag_path(), &[]).await?;
        ff.requester = self.requester.clone();
        Ok(ff)
    }

    /// Set this feature flag to the given state.
    ///
    /// `state` must be one of: `"on"`, `"off"`, `"allowed"`, `"allowed_on"`, `"hidden"`.
    ///
    /// # Canvas API
    /// `PUT /api/v1/:context_type/:context_id/features/flags/:feature`
    pub async fn set_feature_flag(&self, state: &str) -> Result<FeatureFlag> {
        let params = vec![("state".to_string(), state.to_string())];
        let mut ff: FeatureFlag = self.req().put(&self.flag_path(), &params).await?;
        ff.requester = self.requester.clone();
        Ok(ff)
    }
}
