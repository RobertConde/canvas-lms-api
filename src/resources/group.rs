use crate::{http::Requester, pagination::PageStream, resources::collaboration::Collaboration};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A Canvas group within a course or account.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct Group {
    pub id: u64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub followed_by_user: Option<bool>,
    pub join_level: Option<String>,
    pub members_count: Option<u64>,
    pub avatar_url: Option<String>,
    pub course_id: Option<u64>,
    pub role: Option<String>,
    pub group_category_id: Option<u64>,
    pub sis_group_id: Option<String>,
    pub sis_import_id: Option<u64>,
    pub storage_quota_mb: Option<u64>,
    pub permissions: Option<serde_json::Value>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Group {
    /// Stream all collaborations in this group.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id/collaborations`
    pub fn get_collaborations(&self) -> PageStream<Collaboration> {
        let group_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("groups/{group_id}/collaborations"),
            vec![],
            {
                let req = Arc::clone(self.req());
                move |mut c: Collaboration, _| {
                    c.requester = Some(Arc::clone(&req));
                    c
                }
            },
        )
    }
}
