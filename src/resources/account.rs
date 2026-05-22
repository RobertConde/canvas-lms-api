use crate::{
    error::Result,
    http::Requester,
    pagination::PageStream,
    params::wrap_params,
    resources::{
        account_calendar::AccountCalendar,
        content_export::{ContentExport, ContentExportParams},
        content_migration::{ContentMigration, Migrator},
        enrollment_term::{EnrollmentTerm, EnrollmentTermParams},
        external_tool::{ExternalTool, ExternalToolParams},
        feature::{Feature, FeatureFlag},
        grading_standard::{GradingStandard, GradingStandardParams},
        group::{Group, GroupCategory, GroupCategoryParams},
        outcome::{OutcomeGroup, UpdateOutcomeGroupParams},
        role::{Role, RoleParams},
        rubric::{Rubric, RubricParams},
        sis_import::SisImport,
        user::User,
    },
};

/// Parameters for updating an account.
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct UpdateAccountParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_time_zone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_storage_quota_mb: Option<u64>,
}
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A Canvas account (institution, sub-account, or department).
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
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

    // -------------------------------------------------------------------------
    // Enrollment Terms
    // -------------------------------------------------------------------------

    /// Fetch a single enrollment term by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/terms/:id`
    pub async fn get_enrollment_term(&self, term_id: u64) -> Result<EnrollmentTerm> {
        let mut t: EnrollmentTerm = self
            .req()
            .get(&format!("accounts/{}/terms/{term_id}", self.id), &[])
            .await?;
        t.requester = self.requester.clone();
        t.account_id = Some(self.id);
        Ok(t)
    }

    /// Stream all enrollment terms for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/terms`
    pub fn get_enrollment_terms(&self) -> PageStream<EnrollmentTerm> {
        let account_id = self.id;
        let req = Arc::clone(self.req());
        PageStream::new_with_injector(
            req,
            &format!("accounts/{account_id}/terms"),
            vec![],
            move |mut t: EnrollmentTerm, r| {
                t.requester = Some(Arc::clone(&r));
                t.account_id = Some(account_id);
                t
            },
        )
    }

    /// Create an enrollment term on this account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:account_id/terms`
    pub async fn create_enrollment_term(
        &self,
        params: EnrollmentTermParams,
    ) -> Result<EnrollmentTerm> {
        let form = wrap_params("enrollment_term", &params);
        let mut t: EnrollmentTerm = self
            .req()
            .post(&format!("accounts/{}/terms", self.id), &form)
            .await?;
        t.requester = self.requester.clone();
        t.account_id = Some(self.id);
        Ok(t)
    }

    // -------------------------------------------------------------------------
    // Grading Standards
    // -------------------------------------------------------------------------

    /// Stream all grading standards for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/grading_standards`
    pub fn get_grading_standards(&self) -> PageStream<GradingStandard> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("accounts/{}/grading_standards", self.id),
            vec![],
        )
    }

    /// Fetch a single grading standard by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/grading_standards/:grading_standard_id`
    pub async fn get_grading_standard(&self, standard_id: u64) -> Result<GradingStandard> {
        self.req()
            .get(
                &format!("accounts/{}/grading_standards/{standard_id}", self.id),
                &[],
            )
            .await
    }

    /// Create a grading standard on this account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:account_id/grading_standards`
    pub async fn create_grading_standard(
        &self,
        params: GradingStandardParams,
    ) -> Result<GradingStandard> {
        let form = wrap_params("grading_scheme_entry", &params.grading_scheme_entry)
            .into_iter()
            .chain([("title".into(), params.title)])
            .collect::<Vec<_>>();
        self.req()
            .post(&format!("accounts/{}/grading_standards", self.id), &form)
            .await
    }

    // -------------------------------------------------------------------------
    // Roles
    // -------------------------------------------------------------------------

    /// Fetch a single role by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/roles/:id`
    pub async fn get_role(&self, role_id: u64) -> Result<Role> {
        self.req()
            .get(&format!("accounts/{}/roles/{role_id}", self.id), &[])
            .await
    }

    /// Stream all roles for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/roles`
    pub fn get_roles(&self) -> PageStream<Role> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("accounts/{}/roles", self.id),
            vec![],
        )
    }

    /// Create a role on this account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:account_id/roles`
    pub async fn create_role(&self, label: &str, params: RoleParams) -> Result<Role> {
        let mut form = vec![("label".into(), label.to_string())];
        if let Some(base) = &params.base_role_type {
            form.push(("base_role_type".into(), base.clone()));
        }
        if let Some(perms) = &params.permissions {
            form.extend(crate::params::to_canvas_params("permissions", perms));
        }
        self.req()
            .post(&format!("accounts/{}/roles", self.id), &form)
            .await
    }

    /// Deactivate a role by ID.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/accounts/:account_id/roles/:id`
    pub async fn deactivate_role(&self, role_id: u64) -> Result<Role> {
        self.req()
            .delete(&format!("accounts/{}/roles/{role_id}", self.id), &[])
            .await
    }

    /// Activate a previously deactivated role.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:account_id/roles/:id/activate`
    pub async fn activate_role(&self, role_id: u64) -> Result<Role> {
        self.req()
            .post(
                &format!("accounts/{}/roles/{role_id}/activate", self.id),
                &[],
            )
            .await
    }

    /// Update a role.
    ///
    /// # Canvas API
    /// `PUT /api/v1/accounts/:account_id/roles/:id`
    pub async fn update_role(&self, role_id: u64, params: RoleParams) -> Result<Role> {
        let mut form: Vec<(String, String)> = vec![];
        if let Some(perms) = &params.permissions {
            form.extend(crate::params::to_canvas_params("permissions", perms));
        }
        self.req()
            .put(&format!("accounts/{}/roles/{role_id}", self.id), &form)
            .await
    }

    // -------------------------------------------------------------------------
    // Content Exports
    // -------------------------------------------------------------------------

    /// Fetch a single content export by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/content_exports/:id`
    pub async fn get_content_export(&self, export_id: u64) -> Result<ContentExport> {
        self.req()
            .get(
                &format!("accounts/{}/content_exports/{export_id}", self.id),
                &[],
            )
            .await
    }

    /// Stream all content exports for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/content_exports`
    pub fn get_content_exports(&self) -> PageStream<ContentExport> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("accounts/{}/content_exports", self.id),
            vec![],
        )
    }

    /// Create a content export for this account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:account_id/content_exports`
    pub async fn create_content_export(
        &self,
        params: ContentExportParams,
    ) -> Result<ContentExport> {
        let form = vec![
            ("export_type".into(), params.export_type),
            (
                "skip_notifications".into(),
                params.skip_notifications.unwrap_or(false).to_string(),
            ),
        ];
        self.req()
            .post(&format!("accounts/{}/content_exports", self.id), &form)
            .await
    }

    // -------------------------------------------------------------------------
    // Features
    // -------------------------------------------------------------------------

    /// Stream all feature flags for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/features`
    pub fn get_features(&self) -> PageStream<Feature> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("accounts/{}/features", self.id),
            vec![],
        )
    }

    /// Fetch a specific feature flag for this account by feature name.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/features/flags/:feature`
    pub async fn get_feature_flag(&self, feature: &str) -> Result<FeatureFlag> {
        let mut ff: FeatureFlag = self
            .req()
            .get(
                &format!("accounts/{}/features/flags/{feature}", self.id),
                &[],
            )
            .await?;
        ff.requester = self.requester.clone();
        Ok(ff)
    }

    /// List all enabled feature names for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:account_id/features/enabled`
    pub async fn get_enabled_features(&self) -> Result<Vec<String>> {
        self.req()
            .get(&format!("accounts/{}/features/enabled", self.id), &[])
            .await
    }

    /// Update this account.
    ///
    /// # Canvas API
    /// `PUT /api/v1/accounts/:id`
    pub async fn update(&self, params: UpdateAccountParams) -> Result<Account> {
        let form = wrap_params("account", &params);
        let mut a: Account = self
            .req()
            .put(&format!("accounts/{}", self.id), &form)
            .await?;
        a.requester = self.requester.clone();
        Ok(a)
    }

    /// Stream all sub-accounts of this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:id/sub_accounts`
    pub fn get_subaccounts(&self) -> PageStream<Account> {
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("accounts/{}/sub_accounts", self.id),
            vec![],
            |mut a: Account, req| {
                a.requester = Some(Arc::clone(&req));
                a
            },
        )
    }

    /// Create a sub-account under this account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:id/sub_accounts`
    pub async fn create_subaccount(&self, name: &str) -> Result<Account> {
        let params = vec![("account[name]".to_string(), name.to_string())];
        let mut a: Account = self
            .req()
            .post(&format!("accounts/{}/sub_accounts", self.id), &params)
            .await?;
        a.requester = self.requester.clone();
        Ok(a)
    }

    /// Stream all users in this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:id/users`
    pub fn get_users(&self) -> PageStream<User> {
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("accounts/{}/users", self.id),
            vec![],
            |mut u: User, req| {
                u.requester = Some(Arc::clone(&req));
                u
            },
        )
    }

    /// Delete a user from this account.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/accounts/:id/users/:user_id`
    pub async fn delete_user(&self, user_id: u64) -> Result<User> {
        let mut u: User = self
            .req()
            .delete(
                &format!("accounts/{}/users/{user_id}", self.id),
                &[],
            )
            .await?;
        u.requester = self.requester.clone();
        Ok(u)
    }

    /// Stream all courses in this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:id/courses`
    pub fn get_courses(
        &self,
    ) -> PageStream<crate::resources::course::Course> {
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("accounts/{}/courses", self.id),
            vec![],
            move |mut c: crate::resources::course::Course, req| {
                c.requester = Some(Arc::clone(&req));
                c
            },
        )
    }

    /// Stream all groups in this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:id/groups`
    pub fn get_groups(&self) -> PageStream<Group> {
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("accounts/{}/groups", self.id),
            vec![],
            |mut g: Group, req| {
                g.requester = Some(Arc::clone(&req));
                g
            },
        )
    }

    /// Stream all group categories in this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:id/group_categories`
    pub fn get_group_categories(&self) -> PageStream<GroupCategory> {
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("accounts/{}/group_categories", self.id),
            vec![],
            |mut gc: GroupCategory, req| {
                gc.requester = Some(Arc::clone(&req));
                gc
            },
        )
    }

    /// Create a group category in this account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:id/group_categories`
    pub async fn create_group_category(&self, params: GroupCategoryParams) -> Result<GroupCategory> {
        let form = wrap_params("group_category", &params);
        let mut gc: GroupCategory = self
            .req()
            .post(&format!("accounts/{}/group_categories", self.id), &form)
            .await?;
        gc.requester = self.requester.clone();
        Ok(gc)
    }

    /// Stream all admins for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:id/admins`
    pub fn get_admins(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("accounts/{}/admins", self.id),
            vec![],
        )
    }

    /// Create an admin for this account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:id/admins`
    pub async fn create_admin(&self, user_id: u64) -> Result<serde_json::Value> {
        let params = vec![("user_id".to_string(), user_id.to_string())];
        self.req()
            .post(&format!("accounts/{}/admins", self.id), &params)
            .await
    }

    /// Stream all authentication providers for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:id/authentication_providers`
    pub fn get_authentication_providers(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("accounts/{}/authentication_providers", self.id),
            vec![],
        )
    }

    /// Create a new user in this account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:id/users`
    pub async fn create_user(&self, params: &[(String, String)]) -> Result<User> {
        let mut u: User = self
            .req()
            .post(&format!("accounts/{}/users", self.id), params)
            .await?;
        u.requester = self.requester.clone();
        Ok(u)
    }

    /// Stream all reports of a given type for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:id/reports/:report_type`
    pub fn get_reports(&self, report_type: &str) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("accounts/{}/reports/{report_type}", self.id),
            vec![],
        )
    }

    /// Create (run) a report for this account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:id/reports/:report_type`
    pub async fn create_report(
        &self,
        report_type: &str,
        params: &[(String, String)],
    ) -> Result<serde_json::Value> {
        self.req()
            .post(
                &format!("accounts/{}/reports/{report_type}", self.id),
                params,
            )
            .await
    }

    /// Get the status of an outcome import for this account.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:id/outcome_imports/:id`
    pub async fn get_outcome_import_status(&self, import_id: u64) -> Result<serde_json::Value> {
        self.req()
            .get(
                &format!("accounts/{}/outcome_imports/{import_id}", self.id),
                &[],
            )
            .await
    }
}
