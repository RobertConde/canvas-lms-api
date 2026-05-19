use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A Canvas user.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: u64,
    pub name: Option<String>,
    pub sortable_name: Option<String>,
    pub short_name: Option<String>,
    pub sis_user_id: Option<String>,
    pub login_id: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub locale: Option<String>,
    pub last_login: Option<DateTime<Utc>>,
    pub time_zone: Option<String>,
    pub bio: Option<String>,
}

/// The currently authenticated user (extends User with additional fields).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CurrentUser {
    pub id: u64,
    pub name: Option<String>,
    pub sortable_name: Option<String>,
    pub short_name: Option<String>,
    pub sis_user_id: Option<String>,
    pub login_id: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub locale: Option<String>,
    pub last_login: Option<DateTime<Utc>>,
    pub time_zone: Option<String>,
    pub bio: Option<String>,
    pub effective_locale: Option<String>,
}

/// A user display stub (id + name only) used in nested contexts.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserDisplay {
    pub id: u64,
    pub display_name: Option<String>,
    pub avatar_image_url: Option<String>,
    pub html_url: Option<String>,
}

/// Identifies a user by numeric ID or as the currently authenticated user.
pub enum UserId {
    Id(u64),
    Current,
}

impl UserId {
    pub(crate) fn to_path_segment(&self) -> String {
        match self {
            UserId::Id(id) => id.to_string(),
            UserId::Current => "self".to_string(),
        }
    }
}
