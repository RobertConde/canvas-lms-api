use serde::{Deserialize, Serialize};

/// An authentication provider configured for a Canvas account.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthenticationProvider {
    pub id: Option<u64>,
    pub auth_type: Option<String>,
    pub position: Option<u64>,
    pub identifier_format: Option<String>,
    pub log_in_url: Option<String>,
    pub log_out_url: Option<String>,
    pub certificate_fingerprint: Option<String>,
    pub change_password_url: Option<String>,
    pub requested_authn_context: Option<String>,
    pub idp_entity_id: Option<String>,
    pub login_attribute: Option<String>,
    pub sig_alg: Option<String>,
}
