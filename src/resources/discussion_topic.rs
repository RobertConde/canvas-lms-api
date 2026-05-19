use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
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
}
