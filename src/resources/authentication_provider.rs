use crate::{error::Result, http::Requester};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// An authentication provider configured for a Canvas account.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
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

    #[serde(skip)]
    pub(crate) account_id: Option<u64>,
    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl AuthenticationProvider {
    fn provider_id(&self) -> u64 {
        self.id.expect("AuthenticationProvider missing id")
    }

    fn acct_id(&self) -> u64 {
        self.account_id
            .expect("AuthenticationProvider missing account_id")
    }

    /// Delete this authentication provider.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/accounts/:account_id/authentication_providers/:id`
    pub async fn delete(&self) -> Result<()> {
        self.req()
            .delete_void(&format!(
                "accounts/{}/authentication_providers/{}",
                self.acct_id(),
                self.provider_id()
            ))
            .await
    }

    /// Update this authentication provider.
    ///
    /// # Canvas API
    /// `PUT /api/v1/accounts/:account_id/authentication_providers/:id`
    pub async fn update(&self, params: &[(String, String)]) -> Result<AuthenticationProvider> {
        let mut provider: AuthenticationProvider = self
            .req()
            .put(
                &format!(
                    "accounts/{}/authentication_providers/{}",
                    self.acct_id(),
                    self.provider_id()
                ),
                params,
            )
            .await?;
        provider.requester = self.requester.clone();
        provider.account_id = self.account_id;
        Ok(provider)
    }
}
