use serde::{Deserialize, Serialize};

/// A Canvas account (institution, sub-account, or department).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Account {
    pub id: u64,
    pub name: Option<String>,
    pub uuid: Option<String>,
    pub parent_account_id: Option<u64>,
    pub root_account_id: Option<u64>,
    pub workflow_state: Option<String>,
    pub default_storage_quota_mb: Option<u64>,
    pub default_user_storage_quota_mb: Option<u64>,
    pub default_group_storage_quota_mb: Option<u64>,
    pub default_time_zone: Option<String>,
    pub sis_account_id: Option<String>,
    pub integration_id: Option<String>,
    pub lti_guid: Option<String>,
}
