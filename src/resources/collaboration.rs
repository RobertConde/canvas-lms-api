use crate::{http::Requester, pagination::PageStream};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A Canvas collaboration (Google Docs, Etherpad, etc.).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Collaboration {
    pub id: u64,
    pub collaboration_type: Option<String>,
    pub document_id: Option<String>,
    pub user_id: Option<u64>,
    pub context_id: Option<u64>,
    pub context_type: Option<String>,
    pub url: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub description: Option<String>,
    pub title: Option<String>,
    pub user_name: Option<String>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Collaboration {
    fn req(&self) -> &Arc<Requester> {
        self.requester.as_ref().expect("requester not initialized")
    }

    /// List collaborators for this collaboration.
    ///
    /// # Canvas API
    /// `GET /api/v1/collaborations/:id/members`
    pub fn get_collaborators(&self) -> PageStream<Collaborator> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("collaborations/{}/members", self.id),
            vec![],
        )
    }
}

/// A user who is a member of a collaboration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Collaborator {
    pub id: u64,
    #[serde(rename = "type")]
    pub collaborator_type: Option<String>,
    pub name: Option<String>,
}
