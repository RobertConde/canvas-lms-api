use crate::{
    error::{CanvasError, Result},
    http::Requester,
    pagination::PageStream,
    params::wrap_params,
    resources::{
        collaboration::Collaboration, discussion_topic::DiscussionTopic, file::File,
        folder::Folder, page::Page, progress::Progress, user::User,
    },
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Parameters for creating or updating a Canvas group.
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateGroupParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_quota_mb: Option<u64>,
}

/// Parameters for updating a group membership.
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateMembershipParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderator: Option<bool>,
}

/// Parameters for creating or updating a group category.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GroupCategoryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_signup: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_leader: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_group_count: Option<u64>,
}

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
    fn propagate(&self, g: &mut Group) {
        g.requester = self.requester.clone();
    }

    /// Edit this group.
    ///
    /// # Canvas API
    /// `PUT /api/v1/groups/:id`
    pub async fn edit(&self, params: UpdateGroupParams) -> Result<Group> {
        let form = wrap_params("group", &params);
        let mut g: Group = self
            .req()
            .put(&format!("groups/{}", self.id), &form)
            .await?;
        self.propagate(&mut g);
        Ok(g)
    }

    /// Delete this group.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/groups/:id`
    pub async fn delete(&self) -> Result<()> {
        self.req().delete_void(&format!("groups/{}", self.id)).await
    }

    /// Stream all users in this group.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id/users`
    pub fn get_users(&self) -> PageStream<User> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("groups/{}/users", self.id),
            vec![],
        )
    }

    /// Stream all memberships in this group.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id/memberships`
    pub fn get_memberships(&self) -> PageStream<GroupMembership> {
        let group_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("groups/{}/memberships", self.id),
            vec![],
            move |mut m: GroupMembership, req| {
                m.requester = Some(Arc::clone(&req));
                m.group_id = Some(group_id);
                m
            },
        )
    }

    /// Create a membership for a user in this group.
    ///
    /// # Canvas API
    /// `POST /api/v1/groups/:id/memberships`
    pub async fn create_membership(&self, user_id: u64) -> Result<GroupMembership> {
        let params = vec![("user_id".to_string(), user_id.to_string())];
        let mut m: GroupMembership = self
            .req()
            .post(&format!("groups/{}/memberships", self.id), &params)
            .await?;
        m.requester = self.requester.clone();
        m.group_id = Some(self.id);
        Ok(m)
    }

    /// Get the membership for a specific user.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id/users/:user_id/membership`
    pub async fn get_membership(&self, user_id: u64) -> Result<GroupMembership> {
        let mut m: GroupMembership = self
            .req()
            .get(
                &format!("groups/{}/users/{user_id}/membership", self.id),
                &[],
            )
            .await?;
        m.requester = self.requester.clone();
        m.group_id = Some(self.id);
        Ok(m)
    }

    /// Update a membership.
    ///
    /// # Canvas API
    /// `PUT /api/v1/groups/:id/memberships/:membership_id`
    pub async fn update_membership(
        &self,
        membership_id: u64,
        params: UpdateMembershipParams,
    ) -> Result<GroupMembership> {
        let form = wrap_params("membership", &params);
        let mut m: GroupMembership = self
            .req()
            .put(
                &format!("groups/{}/memberships/{membership_id}", self.id),
                &form,
            )
            .await?;
        m.requester = self.requester.clone();
        m.group_id = Some(self.id);
        Ok(m)
    }

    /// Remove a user from this group.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/groups/:id/users/:user_id`
    pub async fn remove_user(&self, user_id: u64) -> Result<()> {
        self.req()
            .delete_void(&format!("groups/{}/users/{user_id}", self.id))
            .await
    }

    /// Invite users to this group.
    ///
    /// # Canvas API
    /// `POST /api/v1/groups/:id/invite`
    pub async fn invite(&self, invitee_ids: &[u64]) -> Result<Vec<GroupMembership>> {
        let params: Vec<(String, String)> = invitee_ids
            .iter()
            .map(|id| ("invitees[]".to_string(), id.to_string()))
            .collect();
        let memberships: Vec<GroupMembership> = self
            .req()
            .post(&format!("groups/{}/invite", self.id), &params)
            .await?;
        let group_id = self.id;
        Ok(memberships
            .into_iter()
            .map(|mut m| {
                m.requester = self.requester.clone();
                m.group_id = Some(group_id);
                m
            })
            .collect())
    }

    /// Stream all files in this group.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id/files`
    pub fn get_files(&self) -> PageStream<File> {
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("groups/{}/files", self.id),
            vec![],
            |mut f: File, req| {
                f.requester = Some(Arc::clone(&req));
                f
            },
        )
    }

    /// Fetch a single file.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id/files/:file_id`
    pub async fn get_file(&self, file_id: u64) -> Result<File> {
        let mut f: File = self
            .req()
            .get(&format!("groups/{}/files/{file_id}", self.id), &[])
            .await?;
        f.requester = self.requester.clone();
        Ok(f)
    }

    /// Stream all folders in this group.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id/folders`
    pub fn get_folders(&self) -> PageStream<Folder> {
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("groups/{}/folders", self.id),
            vec![],
            |mut f: Folder, req| {
                f.requester = Some(Arc::clone(&req));
                f
            },
        )
    }

    /// Fetch a single folder.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id/folders/:folder_id`
    pub async fn get_folder(&self, folder_id: u64) -> Result<Folder> {
        let mut f: Folder = self
            .req()
            .get(&format!("groups/{}/folders/{folder_id}", self.id), &[])
            .await?;
        f.requester = self.requester.clone();
        Ok(f)
    }

    /// Create a folder in this group.
    ///
    /// # Canvas API
    /// `POST /api/v1/groups/:id/folders`
    pub async fn create_folder(&self, name: &str) -> Result<Folder> {
        let params = vec![("name".to_string(), name.to_string())];
        let mut f: Folder = self
            .req()
            .post(&format!("groups/{}/folders", self.id), &params)
            .await?;
        f.requester = self.requester.clone();
        Ok(f)
    }

    /// Stream all pages in this group.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id/pages`
    pub fn get_pages(&self) -> PageStream<Page> {
        let group_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("groups/{}/pages", self.id),
            vec![],
            move |mut p: Page, req| {
                p.requester = Some(Arc::clone(&req));
                p.group_id = Some(group_id);
                p
            },
        )
    }

    /// Fetch a single page by URL slug.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id/pages/:url`
    pub async fn get_page(&self, url_slug: &str) -> Result<Page> {
        let mut p: Page = self
            .req()
            .get(&format!("groups/{}/pages/{url_slug}", self.id), &[])
            .await?;
        p.requester = self.requester.clone();
        p.group_id = Some(self.id);
        Ok(p)
    }

    /// Stream all discussion topics in this group.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id/discussion_topics`
    pub fn get_discussion_topics(&self) -> PageStream<DiscussionTopic> {
        let group_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("groups/{}/discussion_topics", self.id),
            vec![],
            move |mut t: DiscussionTopic, req| {
                t.requester = Some(Arc::clone(&req));
                t.group_id = Some(group_id);
                t
            },
        )
    }

    /// Fetch a single discussion topic.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id/discussion_topics/:topic_id`
    pub async fn get_discussion_topic(&self, topic_id: u64) -> Result<DiscussionTopic> {
        let mut t: DiscussionTopic = self
            .req()
            .get(
                &format!("groups/{}/discussion_topics/{topic_id}", self.id),
                &[],
            )
            .await?;
        t.requester = self.requester.clone();
        t.group_id = Some(self.id);
        Ok(t)
    }

    /// Create a new page in this group.
    ///
    /// # Canvas API
    /// `POST /api/v1/groups/:id/pages`
    pub async fn create_page(&self, params: &[(String, String)]) -> Result<Page> {
        let mut p: Page = self
            .req()
            .post(&format!("groups/{}/pages", self.id), params)
            .await?;
        p.requester = self.requester.clone();
        p.group_id = Some(self.id);
        Ok(p)
    }

    /// Create a new discussion topic in this group.
    ///
    /// # Canvas API
    /// `POST /api/v1/groups/:id/discussion_topics`
    pub async fn create_discussion_topic(
        &self,
        params: &[(String, String)],
    ) -> Result<DiscussionTopic> {
        let group_id = self.id;
        let mut t: DiscussionTopic = self
            .req()
            .post(&format!("groups/{}/discussion_topics", self.id), params)
            .await?;
        t.requester = self.requester.clone();
        t.group_id = Some(group_id);
        Ok(t)
    }

    /// Stream all tabs for this group.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id/tabs`
    pub fn get_tabs(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("groups/{}/tabs", self.id),
            vec![],
        )
    }

    /// Stream all content migrations for this group.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id/content_migrations`
    pub fn get_content_migrations(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("groups/{}/content_migrations", self.id),
            vec![],
        )
    }

    /// Stream all content exports for this group.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id/content_exports`
    pub fn get_content_exports(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("groups/{}/content_exports", self.id),
            vec![],
        )
    }

    /// Preview processed HTML content in this group's context.
    ///
    /// # Canvas API
    /// `POST /api/v1/groups/:id/preview_html`
    pub async fn preview_html(&self, html: &str) -> Result<serde_json::Value> {
        let params = vec![("html".to_string(), html.to_string())];
        self.req()
            .post(&format!("groups/{}/preview_html", self.id), &params)
            .await
    }

    /// Resolve a folder path to the list of folders along the path.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id/folders/by_path/*full_path`
    pub fn resolve_path(&self, full_path: Option<&str>) -> PageStream<Folder> {
        let path = match full_path {
            Some(p) if !p.is_empty() => format!("groups/{}/folders/by_path/{p}", self.id),
            _ => format!("groups/{}/folders/by_path", self.id),
        };
        PageStream::new(Arc::clone(self.req()), &path, vec![])
    }

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

    /// Upload a file to this group's file storage.
    ///
    /// Canvas uses a two-step upload: first POSTing metadata to obtain an upload URL,
    /// then POSTing the file as multipart form data to that URL.
    ///
    /// # Canvas API
    /// `POST /api/v1/groups/:id/files`
    pub async fn upload_file(
        &self,
        request: crate::upload::UploadRequest,
        data: Vec<u8>,
    ) -> Result<File> {
        crate::upload::initiate_and_upload(
            self.req(),
            &format!("groups/{}/files", self.id),
            request,
            data,
        )
        .await
    }
}

/// Membership of a user in a Canvas group.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct GroupMembership {
    pub id: u64,
    pub group_id: Option<u64>,
    pub user_id: Option<u64>,
    pub workflow_state: Option<String>,
    pub moderator: Option<bool>,
    pub just_created: Option<bool>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl GroupMembership {
    fn group_id_or_err(&self) -> Result<u64> {
        self.group_id.ok_or_else(|| CanvasError::BadRequest {
            message: "GroupMembership has no group_id".to_string(),
            errors: vec![],
        })
    }

    /// Update this membership (change moderator status, etc).
    ///
    /// # Canvas API
    /// `PUT /api/v1/groups/:group_id/memberships/:id`
    pub async fn update(&self, params: UpdateMembershipParams) -> Result<GroupMembership> {
        let group_id = self.group_id_or_err()?;
        let form = wrap_params("membership", &params);
        let mut m: GroupMembership = self
            .req()
            .put(&format!("groups/{group_id}/memberships/{}", self.id), &form)
            .await?;
        m.requester = self.requester.clone();
        m.group_id = self.group_id;
        Ok(m)
    }

    /// Remove this membership.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/groups/:group_id/memberships/:id`
    pub async fn remove_self(&self) -> Result<()> {
        let group_id = self.group_id_or_err()?;
        self.req()
            .delete_void(&format!("groups/{group_id}/memberships/{}", self.id))
            .await
    }
}

/// A Canvas group category (collection of groups).
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct GroupCategory {
    pub id: u64,
    pub name: Option<String>,
    pub role: Option<String>,
    pub self_signup: Option<String>,
    pub auto_leader: Option<String>,
    pub context_type: Option<String>,
    pub context_id: Option<u64>,
    pub group_limit: Option<u64>,
    pub groups_count: Option<u64>,
    pub unassigned_users_count: Option<u64>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl GroupCategory {
    /// Update this group category.
    ///
    /// # Canvas API
    /// `PUT /api/v1/group_categories/:id`
    pub async fn update(&self, params: GroupCategoryParams) -> Result<GroupCategory> {
        let form = wrap_params("group_category", &params);
        let mut gc: GroupCategory = self
            .req()
            .put(&format!("group_categories/{}", self.id), &form)
            .await?;
        gc.requester = self.requester.clone();
        Ok(gc)
    }

    /// Delete this group category.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/group_categories/:id`
    pub async fn delete(&self) -> Result<()> {
        self.req()
            .delete_void(&format!("group_categories/{}", self.id))
            .await
    }

    /// Stream all groups in this category.
    ///
    /// # Canvas API
    /// `GET /api/v1/group_categories/:id/groups`
    pub fn get_groups(&self) -> PageStream<Group> {
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("group_categories/{}/groups", self.id),
            vec![],
            |mut g: Group, req| {
                g.requester = Some(Arc::clone(&req));
                g
            },
        )
    }

    /// Stream all users in this category.
    ///
    /// # Canvas API
    /// `GET /api/v1/group_categories/:id/users`
    pub fn get_users(&self) -> PageStream<User> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("group_categories/{}/users", self.id),
            vec![],
        )
    }

    /// Create a group in this category.
    ///
    /// # Canvas API
    /// `POST /api/v1/group_categories/:id/groups`
    pub async fn create_group(&self, name: &str) -> Result<Group> {
        let params = vec![("name".to_string(), name.to_string())];
        let mut g: Group = self
            .req()
            .post(&format!("group_categories/{}/groups", self.id), &params)
            .await?;
        g.requester = self.requester.clone();
        Ok(g)
    }

    /// Assign unassigned members to groups.
    ///
    /// # Canvas API
    /// `POST /api/v1/group_categories/:id/assign_unassigned_members`
    pub async fn assign_members(&self) -> Result<Progress> {
        let mut p: Progress = self
            .req()
            .post(
                &format!("group_categories/{}/assign_unassigned_members", self.id),
                &[],
            )
            .await?;
        p.requester = self.requester.clone();
        Ok(p)
    }
}
