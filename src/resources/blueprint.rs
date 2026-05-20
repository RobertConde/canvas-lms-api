use crate::{error::Result, http::Requester, pagination::PageStream};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A blueprint template for a Canvas master course.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct BlueprintTemplate {
    pub id: u64,
    pub course_id: Option<u64>,
    pub last_export_completed_at: Option<DateTime<Utc>>,
    pub associated_course_count: Option<u64>,
    pub latest_migration: Option<serde_json::Value>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl BlueprintTemplate {
    fn course_id(&self) -> u64 {
        self.course_id.expect("BlueprintTemplate missing course_id")
    }

    /// Start a migration to push content to all associated courses.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/blueprint_templates/:template_id/migrations`
    pub async fn start_migration(&self) -> Result<BlueprintMigration> {
        let mut migration: BlueprintMigration = self
            .req()
            .post(
                &format!(
                    "courses/{}/blueprint_templates/{}/migrations",
                    self.course_id(),
                    self.id
                ),
                &[],
            )
            .await?;
        migration.requester = self.requester.clone();
        Ok(migration)
    }

    /// Stream migrations for this blueprint template.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/blueprint_templates/:template_id/migrations`
    pub fn get_migrations(&self) -> PageStream<BlueprintMigration> {
        let course_id = self.course_id();
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!(
                "courses/{course_id}/blueprint_templates/{}/migrations",
                self.id
            ),
            vec![],
            move |mut m: BlueprintMigration, req| {
                m.requester = Some(Arc::clone(&req));
                m
            },
        )
    }

    /// Fetch a single migration by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/blueprint_templates/:template_id/migrations/:id`
    pub async fn get_migration(&self, migration_id: u64) -> Result<BlueprintMigration> {
        let mut migration: BlueprintMigration = self
            .req()
            .get(
                &format!(
                    "courses/{}/blueprint_templates/{}/migrations/{migration_id}",
                    self.course_id(),
                    self.id
                ),
                &[],
            )
            .await?;
        migration.requester = self.requester.clone();
        Ok(migration)
    }

    /// Stream unsynced changes for this blueprint template.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/blueprint_templates/:template_id/unsynced_changes`
    pub fn get_unsynced_changes(&self) -> PageStream<ChangeRecord> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!(
                "courses/{}/blueprint_templates/{}/unsynced_changes",
                self.course_id(),
                self.id
            ),
            vec![],
        )
    }

    /// Stream associated courses for this blueprint template.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/blueprint_templates/:template_id/associated_courses`
    pub fn get_associated_courses(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!(
                "courses/{}/blueprint_templates/{}/associated_courses",
                self.course_id(),
                self.id
            ),
            vec![],
        )
    }

    /// Add or remove associated courses for this blueprint template.
    ///
    /// Pass `course_ids_to_add` and/or `course_ids_to_remove` as flat params.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/blueprint_templates/:template_id/update_associations`
    pub async fn update_associated_courses(&self, params: &[(String, String)]) -> Result<bool> {
        let result: serde_json::Value = self
            .req()
            .put(
                &format!(
                    "courses/{}/blueprint_templates/{}/update_associations",
                    self.course_id(),
                    self.id
                ),
                params,
            )
            .await?;
        Ok(result
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }
}

/// A migration that pushes blueprint content to associated courses.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct BlueprintMigration {
    pub id: u64,
    pub template_id: Option<u64>,
    pub course_id: Option<u64>,
    pub subscription_id: Option<u64>,
    pub user_id: Option<u64>,
    pub workflow_state: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub exports_started_at: Option<DateTime<Utc>>,
    pub imports_queued_at: Option<DateTime<Utc>>,
    pub imports_completed_at: Option<DateTime<Utc>>,
    pub comment: Option<String>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl BlueprintMigration {
    fn course_id(&self) -> u64 {
        self.course_id
            .expect("BlueprintMigration missing course_id")
    }

    fn template_id(&self) -> u64 {
        self.template_id
            .expect("BlueprintMigration missing template_id")
    }

    /// Stream the change details for this migration.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/blueprint_templates/:template_id/migrations/:id/details`
    pub fn get_details(&self) -> PageStream<ChangeRecord> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!(
                "courses/{}/blueprint_templates/{}/migrations/{}/details",
                self.course_id(),
                self.template_id(),
                self.id
            ),
            vec![],
        )
    }

    /// Stream import details for a migration on an associated (child) course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/blueprint_subscriptions/:subscription_id/migrations/:id/details`
    pub fn get_import_details(&self) -> PageStream<ChangeRecord> {
        let subscription_id = self
            .subscription_id
            .expect("BlueprintMigration missing subscription_id");
        PageStream::new(
            Arc::clone(self.req()),
            &format!(
                "courses/{}/blueprint_subscriptions/{}/migrations/{}/details",
                self.course_id(),
                subscription_id,
                self.id
            ),
            vec![],
        )
    }
}

/// A record of a single content item change in a blueprint migration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangeRecord {
    pub asset_id: Option<u64>,
    pub asset_type: Option<String>,
    pub asset_name: Option<String>,
    pub change_type: Option<String>,
    pub html_url: Option<String>,
    pub locked: Option<bool>,
    pub exceptions: Option<Vec<serde_json::Value>>,
}

/// A blueprint subscription on a child (associated) course.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct BlueprintSubscription {
    pub id: u64,
    pub template_id: Option<u64>,
    pub blueprint_course: Option<serde_json::Value>,
    pub course_id: Option<u64>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl BlueprintSubscription {
    fn course_id(&self) -> u64 {
        self.course_id
            .expect("BlueprintSubscription missing course_id")
    }

    /// Stream imports for this subscription.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/blueprint_subscriptions/:id/migrations`
    pub fn get_imports(&self) -> PageStream<BlueprintMigration> {
        let course_id = self.course_id();
        let sub_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/blueprint_subscriptions/{sub_id}/migrations"),
            vec![],
            move |mut m: BlueprintMigration, req| {
                m.requester = Some(Arc::clone(&req));
                m
            },
        )
    }

    /// Fetch a single import by migration ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/blueprint_subscriptions/:id/migrations/:migration_id`
    pub async fn get_import(&self, migration_id: u64) -> Result<BlueprintMigration> {
        let mut migration: BlueprintMigration = self
            .req()
            .get(
                &format!(
                    "courses/{}/blueprint_subscriptions/{}/migrations/{migration_id}",
                    self.course_id(),
                    self.id
                ),
                &[],
            )
            .await?;
        migration.requester = self.requester.clone();
        Ok(migration)
    }
}
