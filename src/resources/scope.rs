use serde::{Deserialize, Serialize};

/// An API scope available for developer key restriction.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Scope {
    pub resource: Option<String>,
    pub resource_name: Option<String>,
    pub controller: Option<String>,
    pub action: Option<String>,
    pub verb: Option<String>,
    pub scope: Option<String>,
}
