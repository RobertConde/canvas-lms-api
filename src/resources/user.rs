use crate::{
    error::Result,
    http::Requester,
    pagination::PageStream,
    params::wrap_params,
    resources::{
        communication_channel::CommunicationChannel,
        content_migration::{ContentMigration, Migrator},
        course::Course,
        enrollment::Enrollment,
        file::File,
        folder::Folder,
    },
};

/// Parameters for editing a Canvas user.
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct EditUserParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sortable_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
}
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A Canvas user.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct User {
    pub id: u64,
    pub name: Option<String>,
    pub sortable_name: Option<String>,
    pub short_name: Option<String>,
    pub sis_user_id: Option<String>,
    pub login_id: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub locale: Option<String>,
    pub last_login: Option<DateTime<Utc>>,
    pub time_zone: Option<String>,
    pub bio: Option<String>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl User {
    /// Stream all courses for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/courses`
    pub fn get_courses(&self) -> PageStream<Course> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/courses", self.id),
            vec![],
        )
    }

    /// Stream all enrollments for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/enrollments`
    pub fn get_enrollments(&self) -> PageStream<Enrollment> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/enrollments", self.id),
            vec![],
        )
    }

    /// Stream all communication channels for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/communication_channels`
    pub fn get_communication_channels(&self) -> PageStream<CommunicationChannel> {
        let user_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("users/{user_id}/communication_channels"),
            vec![],
            |mut c: CommunicationChannel, req| {
                c.requester = Some(Arc::clone(&req));
                c
            },
        )
    }

    fn propagate(&self, u: &mut User) {
        u.requester = self.requester.clone();
    }

    /// Edit this user's profile.
    ///
    /// # Canvas API
    /// `PUT /api/v1/users/:id`
    pub async fn edit(&self, params: EditUserParams) -> Result<User> {
        let form = wrap_params("user", &params);
        let mut u: User = self.req().put(&format!("users/{}", self.id), &form).await?;
        self.propagate(&mut u);
        Ok(u)
    }

    /// Get this user's profile.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/profile`
    pub async fn get_profile(&self) -> Result<serde_json::Value> {
        self.req()
            .get(&format!("users/{}/profile", self.id), &[])
            .await
    }

    /// Terminate all sessions for this user.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/users/:id/sessions`
    pub async fn terminate_sessions(&self) -> Result<()> {
        self.req()
            .delete_void(&format!("users/{}/sessions", self.id))
            .await
    }

    /// Merge this user into another user.
    ///
    /// # Canvas API
    /// `PUT /api/v1/users/:id/merge_into/:destination_user_id`
    pub async fn merge_into(&self, destination_user_id: u64) -> Result<User> {
        let mut u: User = self
            .req()
            .put(
                &format!("users/{}/merge_into/{destination_user_id}", self.id),
                &[],
            )
            .await?;
        self.propagate(&mut u);
        Ok(u)
    }

    /// Stream all avatar options for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/avatars`
    pub fn get_avatars(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/avatars", self.id),
            vec![],
        )
    }

    /// Stream page views for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/page_views`
    pub fn get_page_views(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/page_views", self.id),
            vec![],
        )
    }

    /// Stream all observees for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/observees`
    pub fn get_observees(&self) -> PageStream<User> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/observees", self.id),
            vec![],
        )
    }

    /// Add an observee by user ID.
    ///
    /// # Canvas API
    /// `PUT /api/v1/users/:id/observees/:observee_id`
    pub async fn add_observee(&self, observee_id: u64) -> Result<User> {
        let mut u: User = self
            .req()
            .put(&format!("users/{}/observees/{observee_id}", self.id), &[])
            .await?;
        self.propagate(&mut u);
        Ok(u)
    }

    /// Remove an observee.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/users/:id/observees/:observee_id`
    pub async fn remove_observee(&self, observee_id: u64) -> Result<User> {
        let mut u: User = self
            .req()
            .delete(&format!("users/{}/observees/{observee_id}", self.id), &[])
            .await?;
        self.propagate(&mut u);
        Ok(u)
    }

    /// Fetch a single observee.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/observees/:observee_id`
    pub async fn show_observee(&self, observee_id: u64) -> Result<User> {
        let mut u: User = self
            .req()
            .get(&format!("users/{}/observees/{observee_id}", self.id), &[])
            .await?;
        self.propagate(&mut u);
        Ok(u)
    }

    /// Stream all observers of this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/observers`
    pub fn get_observers(&self) -> PageStream<User> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/observers", self.id),
            vec![],
        )
    }

    /// Get all custom colors for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/colors`
    pub async fn get_colors(&self) -> Result<serde_json::Value> {
        self.req()
            .get(&format!("users/{}/colors", self.id), &[])
            .await
    }

    /// Get the color for a specific asset.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/colors/:asset_string`
    pub async fn get_color(&self, asset_string: &str) -> Result<serde_json::Value> {
        self.req()
            .get(&format!("users/{}/colors/{asset_string}", self.id), &[])
            .await
    }

    /// Update the color for a specific asset.
    ///
    /// # Canvas API
    /// `PUT /api/v1/users/:id/colors/:asset_string`
    pub async fn update_color(
        &self,
        asset_string: &str,
        hexcode: &str,
    ) -> Result<serde_json::Value> {
        let params = vec![("hexcode".to_string(), hexcode.to_string())];
        self.req()
            .put(&format!("users/{}/colors/{asset_string}", self.id), &params)
            .await
    }

    /// Stream missing submissions for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/missing_submissions`
    pub fn get_missing_submissions(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/missing_submissions", self.id),
            vec![],
        )
    }

    /// Stream all files for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/files`
    pub fn get_files(&self) -> PageStream<File> {
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("users/{}/files", self.id),
            vec![],
            move |mut f: File, req| {
                f.requester = Some(Arc::clone(&req));
                f
            },
        )
    }

    /// Stream all folders for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/folders`
    pub fn get_folders(&self) -> PageStream<Folder> {
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("users/{}/folders", self.id),
            vec![],
            move |mut f: Folder, req| {
                f.requester = Some(Arc::clone(&req));
                f
            },
        )
    }

    /// Create a folder for this user.
    ///
    /// # Canvas API
    /// `POST /api/v1/users/:id/folders`
    pub async fn create_folder(&self, name: &str) -> Result<Folder> {
        let params = vec![("name".to_string(), name.to_string())];
        let mut f: Folder = self
            .req()
            .post(&format!("users/{}/folders", self.id), &params)
            .await?;
        f.requester = self.requester.clone();
        Ok(f)
    }

    /// Get the file storage quota for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/files/quota`
    pub async fn get_file_quota(&self) -> Result<serde_json::Value> {
        self.req()
            .get(&format!("users/{}/files/quota", self.id), &[])
            .await
    }

    /// Stream login information for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/logins`
    pub fn get_user_logins(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/logins", self.id),
            vec![],
        )
    }

    /// Get profile settings for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/settings`
    pub async fn get_settings(&self) -> Result<serde_json::Value> {
        self.req()
            .get(&format!("users/{}/settings", self.id), &[])
            .await
    }

    /// Update profile settings for this user.
    ///
    /// # Canvas API
    /// `PUT /api/v1/users/:id/settings`
    pub async fn update_settings(&self, params: &[(String, String)]) -> Result<serde_json::Value> {
        self.req()
            .put(&format!("users/{}/settings", self.id), params)
            .await
    }

    /// Create a communication channel for this user.
    ///
    /// `address` is the email address, phone number, etc.
    /// `channel_type` is `"email"`, `"sms"`, `"push"`, etc.
    ///
    /// # Canvas API
    /// `POST /api/v1/users/:id/communication_channels`
    pub async fn create_communication_channel(
        &self,
        address: &str,
        channel_type: &str,
    ) -> Result<CommunicationChannel> {
        let params = vec![
            (
                "communication_channel[address]".to_string(),
                address.to_string(),
            ),
            (
                "communication_channel[type]".to_string(),
                channel_type.to_string(),
            ),
        ];
        let mut channel: CommunicationChannel = self
            .req()
            .post(
                &format!("users/{}/communication_channels", self.id),
                &params,
            )
            .await?;
        channel.requester = self.requester.clone();
        Ok(channel)
    }

    /// Create an observer pairing code for this user.
    ///
    /// # Canvas API
    /// `POST /api/v1/users/:id/observer_pairing_codes`
    pub async fn create_pairing_code(&self) -> Result<serde_json::Value> {
        self.req()
            .post(&format!("users/{}/observer_pairing_codes", self.id), &[])
            .await
    }

    /// Stream authentication events (login/logout log) for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/audit/authentication/users/:id`
    pub fn get_authentication_events(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("audit/authentication/users/{}", self.id),
            vec![],
        )
    }

    /// Stream all feature flags for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/features`
    pub fn get_features(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/features", self.id),
            vec![],
        )
    }

    /// List enabled features for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/features/enabled`
    pub async fn get_enabled_features(&self) -> Result<Vec<String>> {
        self.req()
            .get(&format!("users/{}/features/enabled", self.id), &[])
            .await
    }

    /// Export content for this user.
    ///
    /// # Canvas API
    /// `POST /api/v1/users/:id/content_exports`
    pub async fn export_content(&self, export_type: &str) -> Result<serde_json::Value> {
        let params = vec![("export_type".to_string(), export_type.to_string())];
        self.req()
            .post(&format!("users/{}/content_exports", self.id), &params)
            .await
    }

    /// Stream content exports for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/content_exports`
    pub fn get_content_exports(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/content_exports", self.id),
            vec![],
        )
    }

    /// Stream ePortfolios for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/eportfolios`
    pub fn get_eportfolios(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/eportfolios", self.id),
            vec![],
        )
    }

    /// Stream open poll sessions for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/poll_sessions/opened`
    pub fn get_open_poll_sessions(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/poll_sessions/opened", self.id),
            vec![],
        )
    }

    /// Stream closed poll sessions for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/poll_sessions/closed`
    pub fn get_closed_poll_sessions(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/poll_sessions/closed", self.id),
            vec![],
        )
    }

    /// Fetch a single file from this user's files.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/files/:file_id`
    pub async fn get_file(&self, file_id: u64) -> Result<File> {
        let mut f: File = self
            .req()
            .get(&format!("users/{}/files/{file_id}", self.id), &[])
            .await?;
        f.requester = self.requester.clone();
        Ok(f)
    }

    /// Fetch a single folder from this user's folders.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/folders/:folder_id`
    pub async fn get_folder(&self, folder_id: u64) -> Result<Folder> {
        let mut f: Folder = self
            .req()
            .get(&format!("users/{}/folders/{folder_id}", self.id), &[])
            .await?;
        f.requester = self.requester.clone();
        Ok(f)
    }

    /// Resolve a folder path under this user's files root.
    ///
    /// Pass `None` to get the root folder. Pass a path like
    /// `"Folder_Level_1/Folder_Level_2"` to walk the tree.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/folders/by_path/*full_path`
    pub fn resolve_path(&self, full_path: Option<&str>) -> PageStream<Folder> {
        let endpoint = match full_path {
            Some(p) if !p.is_empty() => {
                format!("users/{}/folders/by_path/{p}", self.id)
            }
            _ => format!("users/{}/folders/by_path", self.id),
        };
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &endpoint,
            vec![],
            |mut f: Folder, req| {
                f.requester = Some(Arc::clone(&req));
                f
            },
        )
    }

    /// Stream grade-change audit events where this user is a grader.
    ///
    /// # Canvas API
    /// `GET /api/v1/audit/grade_change/graders/:id`
    pub fn get_grade_change_events_for_grader(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("audit/grade_change/graders/{}", self.id),
            vec![],
        )
    }

    /// Stream grade-change audit events where this user is a student.
    ///
    /// # Canvas API
    /// `GET /api/v1/audit/grade_change/students/:id`
    pub fn get_grade_change_events_for_student(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("audit/grade_change/students/{}", self.id),
            vec![],
        )
    }

    /// Fetch a single content migration for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/content_migrations/:migration_id`
    pub async fn get_content_migration(&self, migration_id: u64) -> Result<ContentMigration> {
        let mut m: ContentMigration = self
            .req()
            .get(
                &format!("users/{}/content_migrations/{migration_id}", self.id),
                &[],
            )
            .await?;
        m.requester = self.requester.clone();
        Ok(m)
    }

    /// Stream all content migrations for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/content_migrations`
    pub fn get_content_migrations(&self) -> PageStream<ContentMigration> {
        let user_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("users/{user_id}/content_migrations"),
            vec![],
            move |mut m: ContentMigration, req| {
                m.requester = Some(Arc::clone(&req));
                m.user_id = Some(user_id);
                m
            },
        )
    }

    /// Create a content migration for this user.
    ///
    /// # Canvas API
    /// `POST /api/v1/users/:id/content_migrations`
    pub async fn create_content_migration(&self, migration_type: &str) -> Result<ContentMigration> {
        let params = vec![("migration_type".to_string(), migration_type.to_string())];
        let mut m: ContentMigration = self
            .req()
            .post(&format!("users/{}/content_migrations", self.id), &params)
            .await?;
        m.requester = self.requester.clone();
        m.user_id = Some(self.id);
        Ok(m)
    }

    /// Stream available migration systems (migrators) for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/content_migrations/migrators`
    pub fn get_migration_systems(&self) -> PageStream<Migrator> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/content_migrations/migrators", self.id),
            vec![],
        )
    }

    /// Fetch a specific feature flag for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/features/flags/:feature`
    pub async fn get_feature_flag(&self, feature: &str) -> Result<serde_json::Value> {
        self.req()
            .get(&format!("users/{}/features/flags/{feature}", self.id), &[])
            .await
    }

    /// Upload a file to this user's file storage.
    ///
    /// Canvas uses a two-step upload: first POSTing metadata to obtain an upload URL,
    /// then POSTing the file as multipart form data to that URL.
    ///
    /// # Canvas API
    /// `POST /api/v1/users/:id/files`
    pub async fn upload_file(
        &self,
        request: crate::upload::UploadRequest,
        data: Vec<u8>,
    ) -> Result<File> {
        crate::upload::initiate_and_upload(
            self.req(),
            &format!("users/{}/files", self.id),
            request,
            data,
        )
        .await
    }

    /// Add an observee via credentials (same endpoint as add_observee but without observee_id).
    ///
    /// # Canvas API
    /// `POST /api/v1/users/:id/observees`
    pub async fn add_observee_with_credentials(&self, params: &[(String, String)]) -> Result<User> {
        let mut u: User = self
            .req()
            .post(&format!("users/{}/observees", self.id), params)
            .await?;
        u.requester = self.requester.clone();
        Ok(u)
    }

    /// Stream calendar events for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/calendar_events`
    pub fn get_calendar_events(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/calendar_events", self.id),
            vec![],
        )
    }

    /// Fetch a single content export by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/content_exports/:id`
    pub async fn get_content_export(&self, export_id: u64) -> Result<serde_json::Value> {
        self.req()
            .get(
                &format!("users/{}/content_exports/{export_id}", self.id),
                &[],
            )
            .await
    }

    /// Stream available content licenses for this user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id/content_licenses`
    pub fn get_licenses(&self) -> PageStream<serde_json::Value> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("users/{}/content_licenses", self.id),
            vec![],
        )
    }

    /// Set usage rights on files for this user.
    ///
    /// # Canvas API
    /// `PUT /api/v1/users/:id/usage_rights`
    pub async fn set_usage_rights(&self, params: &[(String, String)]) -> Result<serde_json::Value> {
        self.req()
            .put(&format!("users/{}/usage_rights", self.id), params)
            .await
    }

    /// Remove usage rights from files for this user.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/users/:id/usage_rights`
    pub async fn remove_usage_rights(
        &self,
        params: &[(String, String)],
    ) -> Result<serde_json::Value> {
        self.req()
            .delete(&format!("users/{}/usage_rights", self.id), params)
            .await
    }
}

/// The currently authenticated user (extends User with additional fields).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CurrentUser {
    pub id: u64,
    pub name: Option<String>,
    pub sortable_name: Option<String>,
    pub short_name: Option<String>,
    pub sis_user_id: Option<String>,
    pub login_id: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub locale: Option<String>,
    pub last_login: Option<DateTime<Utc>>,
    pub time_zone: Option<String>,
    pub bio: Option<String>,
    pub effective_locale: Option<String>,
}

/// A user display stub (id + name only) used in nested contexts.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserDisplay {
    pub id: u64,
    pub display_name: Option<String>,
    pub avatar_image_url: Option<String>,
    pub html_url: Option<String>,
}

/// Identifies a user by numeric ID or as the currently authenticated user.
pub enum UserId {
    Id(u64),
    Current,
}

impl UserId {
    pub(crate) fn to_path_segment(&self) -> String {
        match self {
            UserId::Id(id) => id.to_string(),
            UserId::Current => "self".to_string(),
        }
    }
}
