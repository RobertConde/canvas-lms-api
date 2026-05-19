use crate::{
    error::Result,
    http::Requester,
    pagination::PageStream,
    resources::{
        account::Account,
        course::Course,
        user::{CurrentUser, User, UserId},
    },
};
use reqwest::Client;
use std::sync::Arc;
use url::Url;

/// The Canvas LMS API client. All interaction starts here.
///
/// # Example
/// ```no_run
/// # #[tokio::main] async fn main() -> canvas_lms_api::Result<()> {
/// let canvas = canvas_lms_api::Canvas::new("https://canvas.example.edu", "my-token")?;
/// let course = canvas.get_course(1).await?;
/// println!("{}", course.name.unwrap_or_default());
/// # Ok(()) }
/// ```
pub struct Canvas {
    pub(crate) requester: Arc<Requester>,
}

impl Canvas {
    /// Create a new Canvas client.
    ///
    /// `base_url` should be the institution root (e.g. `https://canvas.example.edu`),
    /// not including `/api/v1`.
    pub fn new(base_url: &str, access_token: &str) -> Result<Self> {
        Self::with_client(base_url, access_token, Client::new())
    }

    /// Create a Canvas client with a custom [`reqwest::Client`] (for proxy, TLS config, etc.).
    pub fn with_client(base_url: &str, access_token: &str, client: Client) -> Result<Self> {
        let base_url = validate_base_url(base_url)?;
        let api_url = base_url.join("api/v1/")?;
        Ok(Self {
            requester: Arc::new(Requester::new(
                api_url,
                access_token.trim().to_string(),
                client,
            )),
        })
    }

    // -------------------------------------------------------------------------
    // Courses
    // -------------------------------------------------------------------------

    /// Fetch a single course by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id`
    pub async fn get_course(&self, course_id: u64) -> Result<Course> {
        let mut course: Course = self
            .requester
            .get(&format!("courses/{course_id}"), &[])
            .await?;
        course.requester = Some(Arc::clone(&self.requester));
        Ok(course)
    }

    /// Stream all courses visible to the authenticated user.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses`
    pub fn get_courses(&self) -> PageStream<Course> {
        PageStream::new(Arc::clone(&self.requester), "courses", vec![])
    }

    // -------------------------------------------------------------------------
    // Users
    // -------------------------------------------------------------------------

    /// Fetch a single user by ID or `UserId::Current` for the authenticated user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id`
    pub async fn get_user(&self, user_id: UserId) -> Result<User> {
        let id = user_id.to_path_segment();
        self.requester.get(&format!("users/{id}"), &[]).await
    }

    /// Fetch the currently authenticated user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/self`
    pub async fn get_current_user(&self) -> Result<CurrentUser> {
        self.requester.get("users/self", &[]).await
    }

    // -------------------------------------------------------------------------
    // Accounts
    // -------------------------------------------------------------------------

    /// Fetch a single account by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:id`
    pub async fn get_account(&self, account_id: u64) -> Result<Account> {
        self.requester
            .get(&format!("accounts/{account_id}"), &[])
            .await
    }

    /// Stream all accounts accessible to the authenticated user.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts`
    pub fn get_accounts(&self) -> PageStream<Account> {
        PageStream::new(Arc::clone(&self.requester), "accounts", vec![])
    }
}

fn validate_base_url(raw: &str) -> Result<Url> {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.contains("/api/v1") {
        return Err(crate::error::CanvasError::ApiError {
            status: 0,
            message: "base_url should not include /api/v1".into(),
        });
    }
    Ok(Url::parse(&format!("{trimmed}/"))?)
}
