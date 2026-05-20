use crate::error::Result;
use crate::http::Requester;
use crate::params::wrap_params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConversationParticipant {
    pub id: u64,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConversationMessage {
    pub id: u64,
    pub created_at: Option<String>,
    pub body: Option<String>,
    pub author_id: Option<u64>,
    pub generated: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct Conversation {
    pub id: u64,
    pub subject: Option<String>,
    pub workflow_state: Option<String>,
    pub last_message: Option<String>,
    pub last_message_at: Option<String>,
    pub message_count: Option<u64>,
    pub subscribed: Option<bool>,
    pub private: Option<bool>,
    pub starred: Option<bool>,
    pub audience: Option<Vec<u64>>,
    pub participants: Option<Vec<ConversationParticipant>>,
    pub messages: Option<Vec<ConversationMessage>>,
    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ConversationParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_new: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_conversation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_code: Option<String>,
}

impl Conversation {
    fn endpoint(&self) -> String {
        format!("conversations/{}", self.id)
    }

    /// Add a message to this conversation.
    ///
    /// `POST /api/v1/conversations/:id/add_message`
    pub async fn add_message(&self, body: &str) -> Result<Conversation> {
        let params = vec![("body".into(), body.to_string())];
        let endpoint = format!("{}/add_message", self.endpoint());
        let mut c: Conversation = self.req().post(&endpoint, &params).await?;
        c.requester = self.requester.clone();
        Ok(c)
    }

    /// Add recipients to this conversation.
    ///
    /// `POST /api/v1/conversations/:id/add_recipients`
    pub async fn add_recipients(&self, recipients: &[&str]) -> Result<Conversation> {
        let params: Vec<(String, String)> = recipients
            .iter()
            .map(|r| ("recipients[]".into(), r.to_string()))
            .collect();
        let endpoint = format!("{}/add_recipients", self.endpoint());
        let mut c: Conversation = self.req().post(&endpoint, &params).await?;
        c.requester = self.requester.clone();
        Ok(c)
    }

    /// Delete this conversation.
    ///
    /// `DELETE /api/v1/conversations/:id`
    pub async fn delete(&self) -> Result<Conversation> {
        let mut c: Conversation = self.req().delete(&self.endpoint(), &[]).await?;
        c.requester = self.requester.clone();
        Ok(c)
    }

    /// Delete specific messages from this conversation.
    ///
    /// `POST /api/v1/conversations/:id/remove_messages`
    pub async fn delete_messages(&self, message_ids: &[u64]) -> Result<serde_json::Value> {
        let params: Vec<(String, String)> = message_ids
            .iter()
            .map(|id| ("remove[]".into(), id.to_string()))
            .collect();
        let endpoint = format!("{}/remove_messages", self.endpoint());
        self.req().post(&endpoint, &params).await
    }

    /// Update this conversation (mark read, star, subscribe, etc.).
    ///
    /// `PUT /api/v1/conversations/:id`
    pub async fn edit(&self, params: &[(String, String)]) -> Result<Conversation> {
        let mut c: Conversation = self.req().put(&self.endpoint(), params).await?;
        c.requester = self.requester.clone();
        Ok(c)
    }

    /// Update this conversation's workflow_state.
    ///
    /// `PUT /api/v1/conversations/:id`
    pub async fn set_workflow_state(&self, state: &str) -> Result<Conversation> {
        let params = wrap_params(
            "conversation",
            &serde_json::json!({ "workflow_state": state }),
        );
        self.edit(&params).await
    }
}
