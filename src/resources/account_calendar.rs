use serde::{Deserialize, Serialize};

/// An account calendar in Canvas.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountCalendar {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub parent_account_id: Option<u64>,
    pub root_account_id: Option<u64>,
    pub visible: Option<bool>,
    pub auto_subscribe: Option<bool>,
    pub sub_account_count: Option<u64>,
    pub calendar_event_url: Option<String>,
    pub can_create_calendar_events: Option<bool>,
    pub create_calendar_event_url: Option<String>,
    pub new_calendar_event_url: Option<String>,
}
