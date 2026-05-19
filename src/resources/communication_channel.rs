use crate::{error::Result, http::Requester};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// A Canvas communication channel (email, SMS, push notification, etc.).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommunicationChannel {
    pub id: u64,
    pub address: Option<String>,
    #[serde(rename = "type")]
    pub channel_type: Option<String>,
    pub position: Option<u64>,
    pub user_id: Option<u64>,
    pub workflow_state: Option<String>,
    pub created_at: Option<DateTime<Utc>>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl CommunicationChannel {
    fn req(&self) -> &Arc<Requester> {
        self.requester.as_ref().expect("requester not initialized")
    }

    fn user_id(&self) -> u64 {
        self.user_id.expect("CommunicationChannel missing user_id")
    }

    /// Delete this communication channel.
    ///
    /// Returns `true` if successfully deleted.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/users/:user_id/communication_channels/:id`
    pub async fn delete(&self) -> Result<bool> {
        let result: Value = self
            .req()
            .delete(
                &format!(
                    "users/{}/communication_channels/{}",
                    self.user_id(),
                    self.id
                ),
                &[],
            )
            .await?;
        Ok(result.get("workflow_state").and_then(|v| v.as_str()) == Some("deleted"))
    }

    /// Fetch the notification preference for a specific notification type.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:user_id/communication_channels/:id/notification_preferences/:notification`
    pub async fn get_preference(&self, notification: &str) -> Result<Value> {
        let data: Value = self
            .req()
            .get(
                &format!(
                    "users/{}/communication_channels/{}/notification_preferences/{}",
                    self.user_id(),
                    self.id,
                    notification
                ),
                &[],
            )
            .await?;
        Ok(data
            .get("notification_preferences")
            .and_then(|a| a.get(0))
            .cloned()
            .unwrap_or(Value::Null))
    }

    /// Fetch all notification preference categories for this channel.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:user_id/communication_channels/:id/notification_preference_categories`
    pub async fn get_preference_categories(&self) -> Result<Vec<String>> {
        let data: Value = self
            .req()
            .get(
                &format!(
                    "users/{}/communication_channels/{}/notification_preference_categories",
                    self.user_id(),
                    self.id
                ),
                &[],
            )
            .await?;
        let cats = data
            .get("categories")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        Ok(cats)
    }

    /// Fetch all notification preferences for this channel.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:user_id/communication_channels/:id/notification_preferences`
    pub async fn get_preferences(&self) -> Result<Vec<Value>> {
        let data: Value = self
            .req()
            .get(
                &format!(
                    "users/{}/communication_channels/{}/notification_preferences",
                    self.user_id(),
                    self.id
                ),
                &[],
            )
            .await?;
        Ok(data
            .get("notification_preferences")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default())
    }

    /// Update the preference for a specific notification on this channel.
    ///
    /// `frequency` can be `"immediately"`, `"daily"`, `"weekly"`, or `"never"`.
    ///
    /// # Canvas API
    /// `PUT /api/v1/users/self/communication_channels/:id/notification_preferences/:notification`
    pub async fn update_preference(&self, notification: &str, frequency: &str) -> Result<Value> {
        let params = vec![(
            "notification_preferences[frequency]".to_string(),
            frequency.to_string(),
        )];
        let data: Value = self
            .req()
            .put(
                &format!(
                    "users/self/communication_channels/{}/notification_preferences/{}",
                    self.id, notification
                ),
                &params,
            )
            .await?;
        Ok(data
            .get("notification_preferences")
            .and_then(|a| a.get(0))
            .cloned()
            .unwrap_or(Value::Null))
    }

    /// Update preferences for all notifications in a category on this channel.
    ///
    /// `frequency` can be `"immediately"`, `"daily"`, `"weekly"`, or `"never"`.
    ///
    /// # Canvas API
    /// `PUT /api/v1/users/self/communication_channels/:id/notification_preference_categories/:category`
    pub async fn update_preferences_by_category(
        &self,
        category: &str,
        frequency: &str,
    ) -> Result<Vec<Value>> {
        let params = vec![(
            "notification_preferences[frequency]".to_string(),
            frequency.to_string(),
        )];
        let data: Value = self
            .req()
            .put(
                &format!(
                    "users/self/communication_channels/{}/notification_preference_categories/{}",
                    self.id, category
                ),
                &params,
            )
            .await?;
        Ok(data
            .get("notification_preferences")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default())
    }
}
