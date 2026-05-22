use crate::{
    error::{CanvasError, Result},
    http::Requester,
    pagination::PageStream,
    params::wrap_params,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Parameters for creating or updating a Canvas discussion topic.
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateDiscussionParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discussion_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delayed_post_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_initial_post: Option<bool>,
}

/// Parameters for posting an entry on a discussion topic.
#[derive(Debug, Default, Clone, Serialize)]
pub struct PostEntryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// A Canvas discussion topic (announcement or discussion board post).
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct DiscussionTopic {
    pub id: u64,
    pub course_id: Option<u64>,
    pub title: Option<String>,
    pub message: Option<String>,
    pub html_url: Option<String>,
    pub posted_at: Option<DateTime<Utc>>,
    pub last_reply_at: Option<DateTime<Utc>>,
    pub require_initial_post: Option<bool>,
    pub user_can_see_posts: Option<bool>,
    pub discussion_subentry_count: Option<u64>,
    pub read_state: Option<String>,
    pub unread_count: Option<u64>,
    pub subscribed: Option<bool>,
    pub discussion_type: Option<String>,
    pub published: Option<bool>,
    pub locked: Option<bool>,
    pub pinned: Option<bool>,
    pub locked_for_user: Option<bool>,
    pub assignment_id: Option<u64>,
    pub delayed_post_at: Option<DateTime<Utc>>,
    pub due_at: Option<DateTime<Utc>>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
    #[serde(skip)]
    pub course_id_ctx: Option<u64>,
    #[serde(skip)]
    pub group_id: Option<u64>,
}

impl DiscussionTopic {
    fn parent_prefix(&self) -> Result<String> {
        if let Some(id) = self.course_id.or(self.course_id_ctx) {
            Ok(format!("courses/{id}"))
        } else if let Some(id) = self.group_id {
            Ok(format!("groups/{id}"))
        } else {
            Err(CanvasError::BadRequest {
                message: "DiscussionTopic has no course_id or group_id".to_string(),
                errors: vec![],
            })
        }
    }

    fn propagate(&self, topic: &mut DiscussionTopic) {
        topic.requester = self.requester.clone();
        topic.course_id_ctx = self.course_id.or(self.course_id_ctx);
        topic.group_id = self.group_id;
    }

    fn make_entry(&self, entry: DiscussionEntry) -> DiscussionEntry {
        let mut e = entry;
        e.requester = self.requester.clone();
        e.course_id = self.course_id.or(self.course_id_ctx);
        e.group_id = self.group_id;
        e.topic_id = Some(self.id);
        e
    }

    /// Update this discussion topic.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/discussion_topics/:id`
    pub async fn update(&self, params: UpdateDiscussionParams) -> Result<DiscussionTopic> {
        let prefix = self.parent_prefix()?;
        let form = wrap_params("discussion_topic", &params);
        let mut topic: DiscussionTopic = self
            .req()
            .put(&format!("{prefix}/discussion_topics/{}", self.id), &form)
            .await?;
        self.propagate(&mut topic);
        Ok(topic)
    }

    /// Delete this discussion topic.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/discussion_topics/:id`
    pub async fn delete(&self) -> Result<()> {
        let prefix = self.parent_prefix()?;
        self.req()
            .delete_void(&format!("{prefix}/discussion_topics/{}", self.id))
            .await
    }

    /// Post a new top-level entry to this discussion topic.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/discussion_topics/:id/entries`
    pub async fn post_entry(&self, params: PostEntryParams) -> Result<DiscussionEntry> {
        let prefix = self.parent_prefix()?;
        let form = wrap_params("discussion_entry", &params);
        let entry: DiscussionEntry = self
            .req()
            .post(
                &format!("{prefix}/discussion_topics/{}/entries", self.id),
                &form,
            )
            .await?;
        Ok(self.make_entry(entry))
    }

    /// Stream all top-level entries of this topic.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/discussion_topics/:id/entries`
    pub fn get_topic_entries(&self) -> PageStream<DiscussionEntry> {
        let course_id = self.course_id.or(self.course_id_ctx);
        let group_id = self.group_id;
        let topic_id = self.id;
        let prefix = if let Some(id) = course_id {
            format!("courses/{id}")
        } else if let Some(id) = group_id {
            format!("groups/{id}")
        } else {
            String::new()
        };
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("{prefix}/discussion_topics/{topic_id}/entries"),
            vec![],
            move |mut e: DiscussionEntry, req| {
                e.requester = Some(Arc::clone(&req));
                e.course_id = course_id;
                e.group_id = group_id;
                e.topic_id = Some(topic_id);
                e
            },
        )
    }

    /// Fetch specific entries by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/discussion_topics/:id/entry_list?ids[]=...`
    pub async fn get_entries(&self, ids: &[u64]) -> Result<Vec<DiscussionEntry>> {
        let prefix = self.parent_prefix()?;
        let params: Vec<(String, String)> = ids
            .iter()
            .map(|id| ("ids[]".to_string(), id.to_string()))
            .collect();
        let entries: Vec<DiscussionEntry> = self
            .req()
            .get(
                &format!("{prefix}/discussion_topics/{}/entry_list", self.id),
                &params,
            )
            .await?;
        Ok(entries.into_iter().map(|e| self.make_entry(e)).collect())
    }

    /// Mark this topic as read.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/discussion_topics/:id/read`
    pub async fn mark_as_read(&self) -> Result<()> {
        let prefix = self.parent_prefix()?;
        self.req()
            .put_void(&format!("{prefix}/discussion_topics/{}/read", self.id))
            .await
    }

    /// Mark this topic as unread.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/discussion_topics/:id/read`
    pub async fn mark_as_unread(&self) -> Result<()> {
        let prefix = self.parent_prefix()?;
        self.req()
            .delete_void(&format!("{prefix}/discussion_topics/{}/read", self.id))
            .await
    }

    /// Mark all entries as read.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/discussion_topics/:id/read_all`
    pub async fn mark_entries_as_read(&self, forced: bool) -> Result<()> {
        let prefix = self.parent_prefix()?;
        let params = if forced {
            vec![("forced_read_state".to_string(), "true".to_string())]
        } else {
            vec![]
        };
        let _ = params; // Canvas ignores this in practice; just issue the PUT
        self.req()
            .put_void(&format!("{prefix}/discussion_topics/{}/read_all", self.id))
            .await
    }

    /// Mark all entries as unread.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/discussion_topics/:id/read_all`
    pub async fn mark_entries_as_unread(&self, forced: bool) -> Result<()> {
        let prefix = self.parent_prefix()?;
        let _ = forced;
        self.req()
            .delete_void(&format!(
                "{prefix}/discussion_topics/{}/read_all",
                self.id
            ))
            .await
    }

    /// Subscribe to this topic.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/discussion_topics/:id/subscribed`
    pub async fn subscribe(&self) -> Result<()> {
        let prefix = self.parent_prefix()?;
        self.req()
            .put_void(&format!(
                "{prefix}/discussion_topics/{}/subscribed",
                self.id
            ))
            .await
    }

    /// Unsubscribe from this topic.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/discussion_topics/:id/subscribed`
    pub async fn unsubscribe(&self) -> Result<()> {
        let prefix = self.parent_prefix()?;
        self.req()
            .delete_void(&format!(
                "{prefix}/discussion_topics/{}/subscribed",
                self.id
            ))
            .await
    }
}

/// A single entry (post) within a Canvas discussion topic.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct DiscussionEntry {
    pub id: u64,
    pub user_id: Option<u64>,
    pub discussion_id: Option<u64>,
    pub parent_id: Option<u64>,
    pub message: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
    #[serde(skip)]
    pub course_id: Option<u64>,
    #[serde(skip)]
    pub group_id: Option<u64>,
    #[serde(skip)]
    pub topic_id: Option<u64>,
}

impl DiscussionEntry {
    fn parent_prefix(&self) -> Result<String> {
        if let Some(id) = self.course_id {
            Ok(format!("courses/{id}"))
        } else if let Some(id) = self.group_id {
            Ok(format!("groups/{id}"))
        } else {
            Err(CanvasError::BadRequest {
                message: "DiscussionEntry has no course_id or group_id".to_string(),
                errors: vec![],
            })
        }
    }

    fn topic_id_or_err(&self) -> Result<u64> {
        self.topic_id.ok_or_else(|| CanvasError::BadRequest {
            message: "DiscussionEntry has no topic_id".to_string(),
            errors: vec![],
        })
    }

    fn propagate(&self, entry: &mut DiscussionEntry) {
        entry.requester = self.requester.clone();
        entry.course_id = self.course_id;
        entry.group_id = self.group_id;
        entry.topic_id = self.topic_id;
    }

    /// Update this entry's message.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/discussion_topics/:topic_id/entries/:id`
    pub async fn update(&self, message: &str) -> Result<DiscussionEntry> {
        let prefix = self.parent_prefix()?;
        let topic_id = self.topic_id_or_err()?;
        let params = vec![("message".to_string(), message.to_string())];
        let mut entry: DiscussionEntry = self
            .req()
            .put(
                &format!("{prefix}/discussion_topics/{topic_id}/entries/{}", self.id),
                &params,
            )
            .await?;
        self.propagate(&mut entry);
        Ok(entry)
    }

    /// Delete this entry.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/discussion_topics/:topic_id/entries/:id`
    pub async fn delete(&self) -> Result<()> {
        let prefix = self.parent_prefix()?;
        let topic_id = self.topic_id_or_err()?;
        self.req()
            .delete_void(&format!(
                "{prefix}/discussion_topics/{topic_id}/entries/{}",
                self.id
            ))
            .await
    }

    /// Post a reply to this entry.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/discussion_topics/:topic_id/entries/:id/replies`
    pub async fn post_reply(&self, message: &str) -> Result<DiscussionEntry> {
        let prefix = self.parent_prefix()?;
        let topic_id = self.topic_id_or_err()?;
        let params = vec![("message".to_string(), message.to_string())];
        let mut entry: DiscussionEntry = self
            .req()
            .post(
                &format!(
                    "{prefix}/discussion_topics/{topic_id}/entries/{}/replies",
                    self.id
                ),
                &params,
            )
            .await?;
        self.propagate(&mut entry);
        Ok(entry)
    }

    /// Stream replies to this entry.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/discussion_topics/:topic_id/entries/:id/replies`
    pub fn get_replies(&self) -> PageStream<DiscussionEntry> {
        let course_id = self.course_id;
        let group_id = self.group_id;
        let topic_id = self.topic_id.unwrap_or(0);
        let entry_id = self.id;
        let prefix = if let Some(id) = course_id {
            format!("courses/{id}")
        } else if let Some(id) = group_id {
            format!("groups/{id}")
        } else {
            String::new()
        };
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("{prefix}/discussion_topics/{topic_id}/entries/{entry_id}/replies"),
            vec![],
            move |mut e: DiscussionEntry, req| {
                e.requester = Some(Arc::clone(&req));
                e.course_id = course_id;
                e.group_id = group_id;
                e.topic_id = Some(topic_id);
                e
            },
        )
    }

    /// Mark this entry as read.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/discussion_topics/:topic_id/entries/:id/read`
    pub async fn mark_as_read(&self) -> Result<()> {
        let prefix = self.parent_prefix()?;
        let topic_id = self.topic_id_or_err()?;
        self.req()
            .put_void(&format!(
                "{prefix}/discussion_topics/{topic_id}/entries/{}/read",
                self.id
            ))
            .await
    }

    /// Mark this entry as unread.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/discussion_topics/:topic_id/entries/:id/read`
    pub async fn mark_as_unread(&self) -> Result<()> {
        let prefix = self.parent_prefix()?;
        let topic_id = self.topic_id_or_err()?;
        self.req()
            .delete_void(&format!(
                "{prefix}/discussion_topics/{topic_id}/entries/{}/read",
                self.id
            ))
            .await
    }

    /// Rate this entry (0 = unrate, 1 = rate).
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/discussion_topics/:topic_id/entries/:id/rating`
    pub async fn rate(&self, rating: u8) -> Result<()> {
        if rating > 1 {
            return Err(CanvasError::BadRequest {
                message: "rating must be 0 or 1".to_string(),
                errors: vec![],
            });
        }
        let prefix = self.parent_prefix()?;
        let topic_id = self.topic_id_or_err()?;
        let params = vec![("rating".to_string(), rating.to_string())];
        self.req()
            .post_void_with_params(
                &format!(
                    "{prefix}/discussion_topics/{topic_id}/entries/{}/rating",
                    self.id
                ),
                &params,
            )
            .await
    }
}
