use crate::error::Result;
use crate::http::Requester;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppointmentGroup {
    pub id: u64,
    pub title: Option<String>,
    pub context_codes: Option<Vec<String>>,
    pub sub_context_codes: Option<Vec<String>>,
    pub workflow_state: Option<String>,
    pub location_name: Option<String>,
    pub location_address: Option<String>,
    pub description: Option<String>,
    pub participant_type: Option<String>,
    pub participant_visibility: Option<String>,
    pub participant_limit: Option<u64>,
    pub appointments_count: Option<u64>,
    pub new_appointments: Option<Vec<serde_json::Value>>,
    pub max_appointments_per_participant: Option<u64>,
    pub min_appointments_per_participant: Option<u64>,
    pub appointments: Option<Vec<serde_json::Value>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct AppointmentGroupParams {
    pub context_codes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_appointments_per_participant: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_appointments_per_participant: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_visibility: Option<String>,
}

impl AppointmentGroup {
    fn req(&self) -> &Arc<Requester> {
        self.requester.as_ref().expect("requester not injected")
    }

    fn endpoint(&self) -> String {
        format!("appointment_groups/{}", self.id)
    }

    /// Delete this appointment group.
    ///
    /// `DELETE /api/v1/appointment_groups/:id`
    pub async fn delete(&self) -> Result<AppointmentGroup> {
        let mut a: AppointmentGroup = self.req().delete(&self.endpoint(), &[]).await?;
        a.requester = self.requester.clone();
        Ok(a)
    }

    /// Update this appointment group.
    ///
    /// `PUT /api/v1/appointment_groups/:id`
    pub async fn edit(&self, params: AppointmentGroupParams) -> Result<AppointmentGroup> {
        let body = serde_json::to_value(&params).unwrap_or_default();
        let flat = crate::params::wrap_params("appointment_group", &body);
        let mut a: AppointmentGroup = self.req().put(&self.endpoint(), &flat).await?;
        a.requester = self.requester.clone();
        Ok(a)
    }
}
