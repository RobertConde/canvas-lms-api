use serde::{Deserialize, Serialize};

/// A navigation tab in a Canvas course or account.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tab {
    pub id: Option<String>,
    pub html_url: Option<String>,
    pub full_url: Option<String>,
    pub label: Option<String>,
    #[serde(rename = "type")]
    pub tab_type: Option<String>,
    pub hidden: Option<bool>,
    pub visibility: Option<String>,
    pub position: Option<u64>,
}
