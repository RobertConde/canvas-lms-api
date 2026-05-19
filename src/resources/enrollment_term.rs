use crate::error::Result;
use crate::http::Requester;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnrollmentTerm {
    pub id: u64,
    pub sis_term_id: Option<String>,
    pub sis_import_id: Option<u64>,
    pub name: Option<String>,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub created_at: Option<String>,
    pub workflow_state: Option<String>,
    pub overrides: Option<serde_json::Value>,
    pub course_count: Option<u64>,
    /// Account that owns this term — set when returned via Account methods.
    pub account_id: Option<u64>,
    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct EnrollmentTermParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sis_term_id: Option<String>,
}

impl EnrollmentTerm {
    fn req(&self) -> &Arc<Requester> {
        self.requester.as_ref().expect("requester not injected")
    }

    fn endpoint(&self) -> String {
        let account_id = self.account_id.unwrap_or(1);
        format!("accounts/{}/terms/{}", account_id, self.id)
    }

    /// Delete this enrollment term.
    ///
    /// `DELETE /api/v1/accounts/:account_id/terms/:id`
    pub async fn delete(&self) -> Result<EnrollmentTerm> {
        let mut t: EnrollmentTerm = self.req().delete(&self.endpoint(), &[]).await?;
        t.requester = self.requester.clone();
        t.account_id = self.account_id;
        Ok(t)
    }

    /// Update this enrollment term.
    ///
    /// `PUT /api/v1/accounts/:account_id/terms/:id`
    pub async fn edit(&self, params: EnrollmentTermParams) -> Result<EnrollmentTerm> {
        let body = serde_json::to_value(&params).unwrap_or_default();
        let flat = crate::params::wrap_params("enrollment_term", &body);
        let mut t: EnrollmentTerm = self.req().put(&self.endpoint(), &flat).await?;
        t.requester = self.requester.clone();
        t.account_id = self.account_id;
        Ok(t)
    }
}
