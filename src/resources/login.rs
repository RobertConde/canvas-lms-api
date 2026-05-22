use crate::{error::Result, http::Requester, pagination::PageStream};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A Canvas user login (pseudonym).
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct Login {
    pub id: u64,
    pub account_id: Option<u64>,
    pub user_id: Option<u64>,
    pub workflow_state: Option<String>,
    pub unique_id: Option<String>,
    pub sis_user_id: Option<String>,
    pub integration_id: Option<String>,
    pub authentication_provider_id: Option<u64>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Login {
    fn account_id(&self) -> u64 {
        self.account_id.expect("Login missing account_id")
    }

    fn user_id(&self) -> u64 {
        self.user_id.expect("Login missing user_id")
    }

    /// Update this login.
    ///
    /// # Canvas API
    /// `PUT /api/v1/accounts/:account_id/logins/:id`
    pub async fn edit(&self, params: &[(String, String)]) -> Result<Login> {
        let mut login: Login = self
            .req()
            .put(
                &format!("accounts/{}/logins/{}", self.account_id(), self.id),
                params,
            )
            .await?;
        login.requester = self.requester.clone();
        Ok(login)
    }

    /// Delete this login.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/users/:user_id/logins/:id`
    pub async fn delete(&self) -> Result<Login> {
        let mut login: Login = self
            .req()
            .delete(&format!("users/{}/logins/{}", self.user_id(), self.id), &[])
            .await?;
        login.requester = self.requester.clone();
        Ok(login)
    }

    /// Stream authentication events for this login.
    ///
    /// # Canvas API
    /// `GET /api/v1/audit/authentication/logins/:id`
    pub fn get_authentication_events(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("audit/authentication/logins/{}", self.id),
            vec![],
        )
    }
}
