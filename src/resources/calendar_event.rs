use crate::error::Result;
use crate::http::Requester;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CalendarEvent {
    pub id: u64,
    pub title: Option<String>,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub description: Option<String>,
    pub location_name: Option<String>,
    pub location_address: Option<String>,
    pub context_code: Option<String>,
    pub effective_context_code: Option<String>,
    pub workflow_state: Option<String>,
    pub hidden: Option<bool>,
    pub child_events_count: Option<u64>,
    pub all_day: Option<bool>,
    pub all_day_date: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CalendarEventParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_address: Option<String>,
}

impl CalendarEvent {
    fn req(&self) -> &Arc<Requester> {
        self.requester.as_ref().expect("requester not injected")
    }

    fn endpoint(&self) -> String {
        format!("calendar_events/{}", self.id)
    }

    /// Delete this calendar event.
    ///
    /// `DELETE /api/v1/calendar_events/:id`
    pub async fn delete(&self) -> Result<CalendarEvent> {
        let mut e: CalendarEvent = self.req().delete(&self.endpoint(), &[]).await?;
        e.requester = self.requester.clone();
        Ok(e)
    }

    /// Update this calendar event.
    ///
    /// `PUT /api/v1/calendar_events/:id`
    pub async fn edit(&self, params: CalendarEventParams) -> Result<CalendarEvent> {
        let body = serde_json::to_value(&params).unwrap_or_default();
        let flat = crate::params::wrap_params("calendar_event", &body);
        let mut e: CalendarEvent = self.req().put(&self.endpoint(), &flat).await?;
        e.requester = self.requester.clone();
        Ok(e)
    }
}
