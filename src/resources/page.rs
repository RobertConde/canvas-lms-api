use crate::{
    error::{CanvasError, Result},
    http::Requester,
    pagination::PageStream,
    params::wrap_params,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Parameters for creating or updating a Canvas wiki page.
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdatePageParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editing_roles: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub front_page: Option<bool>,
}

/// A Canvas wiki page within a course or group.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct Page {
    pub page_id: Option<u64>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub hide_from_students: Option<bool>,
    pub editing_roles: Option<String>,
    pub last_edited_by: Option<serde_json::Value>,
    pub body: Option<String>,
    pub published: Option<bool>,
    pub front_page: Option<bool>,
    pub locked_for_user: Option<bool>,
    pub lock_info: Option<serde_json::Value>,
    pub lock_explanation: Option<String>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
    /// Injected by `Course::get_pages()` / `Course::get_page()`.
    #[serde(skip)]
    pub course_id: Option<u64>,
    /// Injected by `Group::get_pages()` / `Group::get_page()`.
    #[serde(skip)]
    pub group_id: Option<u64>,
}

impl Page {
    fn parent_prefix(&self) -> Result<String> {
        if let Some(id) = self.course_id {
            Ok(format!("courses/{id}"))
        } else if let Some(id) = self.group_id {
            Ok(format!("groups/{id}"))
        } else {
            Err(CanvasError::BadRequest {
                message: "Page does not have a course_id or group_id".to_string(),
                errors: vec![],
            })
        }
    }

    fn url_slug(&self) -> &str {
        self.url.as_deref().unwrap_or("")
    }

    fn propagate_context(&self, page: &mut Page) {
        page.requester = self.requester.clone();
        page.course_id = self.course_id;
        page.group_id = self.group_id;
    }

    fn propagate_rev_context(&self, rev: &mut PageRevision) {
        rev.requester = self.requester.clone();
        rev.course_id = self.course_id;
        rev.group_id = self.group_id;
    }

    /// Fetch the parent Course or Group that owns this page.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id` or `GET /api/v1/groups/:id`
    pub async fn get_parent(&self) -> Result<serde_json::Value> {
        let prefix = self.parent_prefix()?;
        self.req().get(&prefix, &[]).await
    }

    /// Update this page's title, body, or settings.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/pages/:url`
    pub async fn edit(&self, params: UpdatePageParams) -> Result<Page> {
        let prefix = self.parent_prefix()?;
        let form = wrap_params("wiki_page", &params);
        let mut page: Page = self
            .req()
            .put(&format!("{prefix}/pages/{}", self.url_slug()), &form)
            .await?;
        self.propagate_context(&mut page);
        Ok(page)
    }

    /// Delete this page.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/pages/:url`
    pub async fn delete(&self) -> Result<Page> {
        let prefix = self.parent_prefix()?;
        let mut page: Page = self
            .req()
            .delete(&format!("{prefix}/pages/{}", self.url_slug()), &[])
            .await?;
        self.propagate_context(&mut page);
        Ok(page)
    }

    /// Stream all revisions of this page.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/pages/:url/revisions`
    pub fn get_revisions(&self) -> Result<PageStream<PageRevision>> {
        let prefix = self.parent_prefix()?;
        let slug = self.url_slug().to_string();
        let course_id = self.course_id;
        let group_id = self.group_id;
        Ok(PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("{prefix}/pages/{slug}/revisions"),
            vec![],
            move |mut r: PageRevision, req| {
                r.requester = Some(Arc::clone(&req));
                r.course_id = course_id;
                r.group_id = group_id;
                r
            },
        ))
    }

    /// Fetch a specific revision by revision ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/pages/:url/revisions/:revision_id`
    pub async fn get_revision_by_id(&self, revision_id: u64) -> Result<PageRevision> {
        let prefix = self.parent_prefix()?;
        let mut rev: PageRevision = self
            .req()
            .get(
                &format!("{prefix}/pages/{}/revisions/{revision_id}", self.url_slug()),
                &[],
            )
            .await?;
        self.propagate_rev_context(&mut rev);
        Ok(rev)
    }

    /// Fetch the most recent revision of this page.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/pages/:url/revisions/latest`
    pub async fn show_latest_revision(&self) -> Result<PageRevision> {
        let prefix = self.parent_prefix()?;
        let mut rev: PageRevision = self
            .req()
            .get(
                &format!("{prefix}/pages/{}/revisions/latest", self.url_slug()),
                &[],
            )
            .await?;
        self.propagate_rev_context(&mut rev);
        Ok(rev)
    }

    /// Revert this page to a specific revision.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/pages/:url/revisions/:revision_id`
    pub async fn revert_to_revision(&self, revision_id: u64) -> Result<PageRevision> {
        let prefix = self.parent_prefix()?;
        let mut rev: PageRevision = self
            .req()
            .post(
                &format!("{prefix}/pages/{}/revisions/{revision_id}", self.url_slug()),
                &[],
            )
            .await?;
        self.propagate_rev_context(&mut rev);
        Ok(rev)
    }
}

/// A historical revision of a Canvas wiki page.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageRevision {
    pub revision_id: Option<u64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub latest: Option<bool>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub edited_by: Option<serde_json::Value>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
    #[serde(skip)]
    pub course_id: Option<u64>,
    #[serde(skip)]
    pub group_id: Option<u64>,
}
