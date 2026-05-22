use crate::{error::Result, http::Requester, pagination::PageStream};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A Canvas content migration job.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct ContentMigration {
    pub id: u64,
    pub migration_type: Option<String>,
    pub migration_type_title: Option<String>,
    pub course_id: Option<u64>,
    pub account_id: Option<u64>,
    pub group_id: Option<u64>,
    pub user_id: Option<u64>,
    pub workflow_state: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub pre_attachment: Option<serde_json::Value>,
    pub progress_url: Option<String>,
    pub migration_issues_url: Option<String>,
    pub migration_issues_count: Option<u64>,
    pub attachment: Option<serde_json::Value>,
    pub settings: Option<serde_json::Value>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl ContentMigration {
    fn parent_type(&self) -> &'static str {
        if self.course_id.is_some() {
            "course"
        } else if self.group_id.is_some() {
            "group"
        } else if self.account_id.is_some() {
            "account"
        } else {
            "user"
        }
    }

    fn parent_id(&self) -> u64 {
        self.course_id
            .or(self.group_id)
            .or(self.account_id)
            .or(self.user_id)
            .expect("ContentMigration missing parent id")
    }

    /// Fetch a single migration issue.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/content_migrations/:id/migration_issues/:issue_id`
    pub async fn get_migration_issue(&self, issue_id: u64) -> Result<MigrationIssue> {
        let mut issue: MigrationIssue = self
            .req()
            .get(
                &format!(
                    "{}s/{}/content_migrations/{}/migration_issues/{issue_id}",
                    self.parent_type(),
                    self.parent_id(),
                    self.id
                ),
                &[],
            )
            .await?;
        issue.requester = self.requester.clone();
        Ok(issue)
    }

    /// Stream all migration issues for this migration.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/content_migrations/:id/migration_issues`
    pub fn get_migration_issues(&self) -> PageStream<MigrationIssue> {
        let parent_type = self.parent_type();
        let parent_id = self.parent_id();
        let migration_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!(
                "{parent_type}s/{parent_id}/content_migrations/{migration_id}/migration_issues"
            ),
            vec![],
            |mut issue: MigrationIssue, req| {
                issue.requester = Some(Arc::clone(&req));
                issue
            },
        )
    }

    /// Fetch the progress object for this content migration.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/content_migrations/:id/progress`
    pub async fn get_progress(&self) -> Result<crate::resources::progress::Progress> {
        self.req()
            .get(
                &format!(
                    "{}s/{}/content_migrations/{}/progress",
                    self.parent_type(),
                    self.parent_id(),
                    self.id
                ),
                &[],
            )
            .await
    }

    /// Update this content migration.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/content_migrations/:id`
    pub async fn update(&self, params: &[(String, String)]) -> Result<ContentMigration> {
        let mut migration: ContentMigration = self
            .req()
            .put(
                &format!(
                    "{}s/{}/content_migrations/{}",
                    self.parent_type(),
                    self.parent_id(),
                    self.id
                ),
                params,
            )
            .await?;
        migration.requester = self.requester.clone();
        Ok(migration)
    }
}

/// An issue encountered during a content migration.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct MigrationIssue {
    pub id: u64,
    pub content_migration_url: Option<String>,
    pub description: Option<String>,
    pub workflow_state: Option<String>,
    pub fix_issue_html_url: Option<String>,
    pub issue_type: Option<String>,
    pub error_report_html_url: Option<String>,
    pub error_message: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl MigrationIssue {
    /// Update the workflow state of this migration issue.
    ///
    /// `workflow_state` should be `"active"` or `"resolved"`.
    ///
    /// # Canvas API
    /// `PUT /api/v1/.../content_migrations/:migration_id/migration_issues/:id`
    pub async fn update(&self, workflow_state: &str) -> Result<MigrationIssue> {
        let migration_url = self
            .content_migration_url
            .as_deref()
            .expect("MigrationIssue missing content_migration_url");
        let params = vec![("workflow_state".to_string(), workflow_state.to_string())];
        // Canvas returns content_migration_url as "/api/v1/..." — strip that prefix
        // so Requester can join the relative path to base_url without doubling it.
        let raw = format!("{}/migration_issues/{}", migration_url, self.id);
        let endpoint = raw.trim_start_matches('/');
        let endpoint = endpoint.strip_prefix("api/v1/").unwrap_or(endpoint);
        let mut issue: MigrationIssue = self.req().put(endpoint, &params).await?;
        issue.requester = self.requester.clone();
        Ok(issue)
    }
}

/// Metadata about an available content migration type.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Migrator {
    pub r#type: Option<String>,
    pub requires_file_upload: Option<bool>,
    pub name: Option<String>,
    pub links: Option<serde_json::Value>,
}
