use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeatureFlag {
    pub feature: Option<String>,
    pub context_type: Option<String>,
    pub context_id: Option<u64>,
    /// One of: `"on"`, `"off"`, `"allowed"`, `"allowed_on"`, `"hidden"`
    pub state: Option<String>,
    pub locked: Option<bool>,
    pub transitions: Option<serde_json::Value>,
}
