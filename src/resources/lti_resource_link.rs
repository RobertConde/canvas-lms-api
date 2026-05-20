use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize)]
pub struct CreateLtiResourceLinkParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<serde_json::Value>,
}

/// An LTI resource link in a Canvas course.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LtiResourceLink {
    pub id: u64,
    pub url: Option<String>,
    pub title: Option<String>,
    pub custom: Option<serde_json::Value>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
