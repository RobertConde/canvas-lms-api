use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RolePermission {
    pub enabled: Option<bool>,
    pub locked: Option<bool>,
    pub applicable: Option<bool>,
    pub readonly: Option<bool>,
    pub explicit: Option<bool>,
    pub prior_default: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Role {
    pub id: Option<u64>,
    pub role: Option<String>,
    pub label: Option<String>,
    pub base_role_type: Option<String>,
    pub account: Option<serde_json::Value>,
    pub workflow_state: Option<String>,
    pub permissions: Option<std::collections::HashMap<String, RolePermission>>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RoleParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_role_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<serde_json::Value>,
}
