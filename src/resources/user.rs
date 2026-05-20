use crate::{
    error::Result,
    http::Requester,
    pagination::PageStream,
    resources::{
        communication_channel::CommunicationChannel, course::Course, enrollment::Enrollment,
    },
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A Canvas user.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
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

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl User {
    /// Stream all courses for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/courses`
    pub fn get_courses(&self) -> PageStream<Course> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/courses", self.id),
            vec![],
        )
    }

    /// Stream all enrollments for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/enrollments`
    pub fn get_enrollments(&self) -> PageStream<Enrollment> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/enrollments", self.id),
            vec![],
        )
    }

    /// Stream all communication channels for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/communication_channels`
    pub fn get_communication_channels(&self) -> PageStream<CommunicationChannel> {
        let user_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("users/{user_id}/communication_channels"),
            vec![],
            |mut c: CommunicationChannel, req| {
                c.requester = Some(Arc::clone(&req));
                c
            },
        )
    }

    /// Create a communication channel for this user.
    ///
    /// `address` is the email address, phone number, etc.
    /// `channel_type` is `"email"`, `"sms"`, `"push"`, etc.
    ///
    /// # Canvas API
    /// `POST /api/v1/users/:id/communication_channels`
    pub async fn create_communication_channel(
        &self,
        address: &str,
        channel_type: &str,
    ) -> Result<CommunicationChannel> {
        let params = vec![
            (
                "communication_channel[address]".to_string(),
                address.to_string(),
            ),
            (
                "communication_channel[type]".to_string(),
                channel_type.to_string(),
            ),
        ];
        let mut channel: CommunicationChannel = self
            .req()
            .post(
                &format!("users/{}/communication_channels", self.id),
                &params,
            )
            .await?;
        channel.requester = self.requester.clone();
        Ok(channel)
    }
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
