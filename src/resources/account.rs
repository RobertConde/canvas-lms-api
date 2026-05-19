use crate::{
    error::Result,
    http::Requester,
    pagination::PageStream,
    params::wrap_params,
    resources::{
        account_calendar::AccountCalendar,
        content_migration::{ContentMigration, Migrator},
        external_tool::{ExternalTool, ExternalToolParams},
        outcome::{OutcomeGroup, UpdateOutcomeGroupParams},
        rubric::{Rubric, RubricParams},
        sis_import::SisImport,
    },
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A Canvas account (institution, sub-account, or department).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Account {
    pub id: u64,
    pub name: Option<String>,
    pub uuid: Option<String>,
    pub parent_account_id: Option<u64>,
    pub root_account_id: Option<u64>,
    pub workflow_state: Option<String>,
    pub default_storage_quota_mb: Option<u64>,
    pub default_user_storage_quota_mb: Option<u64>,
    pub default_group_storage_quota_mb: Option<u64>,
    pub default_time_zone: Option<String>,
    pub sis_account_id: Option<String>,
    pub integration_id: Option<String>,
    pub lti_guid: Option<String>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Account {
    fn req(&self) -> &Arc<Requester> {
        self.requester.as_ref().expect("requester not initialized")
    }

    // -------------------------------------------------------------------------
    // Account Calendars
    // -------------------------------------------------------------------------

    /// Fetch this account's calendar.
    ///
    /// # Canvas API
    /// `GET /api/v1/account_calendars/:account_id`
    pub async fn get_account_calendar(&self) -> Result<AccountCalendar> {
        self.req()
            .get(&format!("account_calendars/{}", self.id), &[])
            .await
    }

    /// Stream all account calendars visible under this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/account_calendars`
    pub fn get_all_account_calendars(&self) -> PageStream<AccountCalendar> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("accounts/{}/account_calendars", self.id),
            vec![],
        )
    }

    // -------------------------------------------------------------------------
    // External Tools
    // -------------------------------------------------------------------------

    /// Fetch a single external tool by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/external_tools/:id`
    pub async fn get_external_tool(&self, tool_id: u64) -> Result<ExternalTool> {
        let mut tool: ExternalTool = self
            .req()
            .get(
                &format!("accounts/{}/external_tools/{tool_id}", self.id),
                &[],
            )
            .await?;
        tool.requester = self.requester.clone();
        Ok(tool)
    }

    /// Stream all external tools for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/external_tools`
    pub fn get_external_tools(&self) -> PageStream<ExternalTool> {
        let account_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("accounts/{account_id}/external_tools"),
            vec![],
            |mut t: ExternalTool, req| {
                t.requester = Some(Arc::clone(&req));
                t
            },
        )
    }

    /// Create an external tool on this account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:account_id/external_tools`
    pub async fn create_external_tool(&self, params: ExternalToolParams) -> Result<ExternalTool> {
        let form = wrap_params("external_tool", &params);
        let mut tool: ExternalTool = self
            .req()
            .post(&format!("accounts/{}/external_tools", self.id), &form)
            .await?;
        tool.requester = self.requester.clone();
        Ok(tool)
    }

    // -------------------------------------------------------------------------
    // SIS Imports
    // -------------------------------------------------------------------------

    /// Fetch a single SIS import by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/sis_imports/:id`
    pub async fn get_sis_import(&self, import_id: u64) -> Result<SisImport> {
        let mut import: SisImport = self
            .req()
            .get(
                &format!("accounts/{}/sis_imports/{import_id}", self.id),
                &[],
            )
            .await?;
        import.requester = self.requester.clone();
        Ok(import)
    }

    /// Stream all SIS imports for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/sis_imports`
    pub fn get_sis_imports(&self) -> PageStream<SisImport> {
        let account_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("accounts/{account_id}/sis_imports"),
            vec![],
            |mut i: SisImport, req| {
                i.requester = Some(Arc::clone(&req));
                i
            },
        )
    }

    /// Stream currently running SIS imports for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/sis_imports/importing`
    pub fn get_sis_imports_running(&self) -> PageStream<SisImport> {
        let account_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("accounts/{account_id}/sis_imports/importing"),
            vec![],
            |mut i: SisImport, req| {
                i.requester = Some(Arc::clone(&req));
                i
            },
        )
    }

    /// Abort all pending SIS imports for this account.
    ///
    /// # Canvas API
    /// `PUT /api/v1/accounts/:account_id/sis_imports/abort_all_pending`
    pub async fn abort_sis_imports_pending(&self) -> Result<bool> {
        let result: serde_json::Value = self
            .req()
            .put(
                &format!("accounts/{}/sis_imports/abort_all_pending", self.id),
                &[],
            )
            .await?;
        Ok(result
            .get("aborted")
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }

    // -------------------------------------------------------------------------
    // Rubrics
    // -------------------------------------------------------------------------

    /// Fetch a single rubric by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/rubrics/:id`
    pub async fn get_rubric(&self, rubric_id: u64) -> Result<Rubric> {
        let mut rubric: Rubric = self
            .req()
            .get(&format!("accounts/{}/rubrics/{rubric_id}", self.id), &[])
            .await?;
        rubric.requester = self.requester.clone();
        Ok(rubric)
    }

    /// Stream all rubrics for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/rubrics`
    pub fn get_rubrics(&self) -> PageStream<Rubric> {
        let account_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("accounts/{account_id}/rubrics"),
            vec![],
            |mut r: Rubric, req| {
                r.requester = Some(Arc::clone(&req));
                r
            },
        )
    }

    /// Create a rubric on this account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:account_id/rubrics`
    pub async fn create_rubric(&self, params: RubricParams) -> Result<Rubric> {
        let form = wrap_params("rubric", &params);
        let mut rubric: Rubric = self
            .req()
            .post(&format!("accounts/{}/rubrics", self.id), &form)
            .await?;
        rubric.requester = self.requester.clone();
        Ok(rubric)
    }

    // -------------------------------------------------------------------------
    // Outcome Groups
    // -------------------------------------------------------------------------

    /// Fetch a single outcome group by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/outcome_groups/:id`
    pub async fn get_outcome_group(&self, group_id: u64) -> Result<OutcomeGroup> {
        let mut group: OutcomeGroup = self
            .req()
            .get(
                &format!("accounts/{}/outcome_groups/{group_id}", self.id),
                &[],
            )
            .await?;
        group.requester = self.requester.clone();
        Ok(group)
    }

    /// Stream all outcome group links for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/outcome_group_links`
    pub fn get_outcome_group_links(&self) -> PageStream<crate::resources::outcome::OutcomeLink> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("accounts/{}/outcome_group_links", self.id),
            vec![],
        )
    }

    /// Create a top-level outcome group on this account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:account_id/outcome_groups`
    pub async fn create_outcome_group(
        &self,
        params: UpdateOutcomeGroupParams,
    ) -> Result<OutcomeGroup> {
        let form = wrap_params("outcome_group", &params);
        let mut group: OutcomeGroup = self
            .req()
            .post(&format!("accounts/{}/outcome_groups", self.id), &form)
            .await?;
        group.requester = self.requester.clone();
        Ok(group)
    }

    // -------------------------------------------------------------------------
    // Content Migrations
    // -------------------------------------------------------------------------

    /// Fetch a single content migration by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/content_migrations/:id`
    pub async fn get_content_migration(&self, migration_id: u64) -> Result<ContentMigration> {
        let mut migration: ContentMigration = self
            .req()
            .get(
                &format!("accounts/{}/content_migrations/{migration_id}", self.id),
                &[],
            )
            .await?;
        migration.requester = self.requester.clone();
        Ok(migration)
    }

    /// Stream all content migrations for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/content_migrations`
    pub fn get_content_migrations(&self) -> PageStream<ContentMigration> {
        let account_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("accounts/{account_id}/content_migrations"),
            vec![],
            |mut m: ContentMigration, req| {
                m.requester = Some(Arc::clone(&req));
                m
            },
        )
    }

    /// Create a content migration on this account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:account_id/content_migrations`
    pub async fn create_content_migration(
        &self,
        migration_type: &str,
        params: &[(String, String)],
    ) -> Result<ContentMigration> {
        let mut form = vec![("migration_type".to_string(), migration_type.to_string())];
        form.extend_from_slice(params);
        let mut migration: ContentMigration = self
            .req()
            .post(&format!("accounts/{}/content_migrations", self.id), &form)
            .await?;
        migration.requester = self.requester.clone();
        Ok(migration)
    }

    /// Stream available content migration types for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/content_migrations/migrators`
    pub fn get_migrators(&self) -> PageStream<Migrator> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("accounts/{}/content_migrations/migrators", self.id),
            vec![],
        )
    }
}
