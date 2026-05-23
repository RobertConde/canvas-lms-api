use serde::{Deserialize, Serialize};

/// Usage rights for a Canvas file or set of files.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UsageRights {
    pub use_justification: Option<String>,
    pub license: Option<String>,
    pub license_name: Option<String>,
    pub message: Option<String>,
    pub freely_available: Option<bool>,
    pub public_domain: Option<bool>,
}
