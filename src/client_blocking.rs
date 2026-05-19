use crate::{
    client::Canvas,
    error::{CanvasError, Result},
    resources::{
        account::Account,
        course::Course,
        params::{course_params::CreateCourseParams, user_params::CreateUserParams},
        user::{CurrentUser, User, UserId},
    },
};
use tokio::runtime::Runtime;

/// A synchronous wrapper around [`Canvas`] that blocks on async operations.
///
/// Create one with [`CanvasBlocking::new`] and call methods directly without `.await`.
///
/// Paginated methods (e.g. [`get_courses`][CanvasBlocking::get_courses]) collect all pages
/// eagerly and return `Result<Vec<T>>`.
///
/// # Example
/// ```no_run
/// # fn main() -> canvas_lms_api::Result<()> {
/// let canvas = canvas_lms_api::CanvasBlocking::new("https://canvas.example.edu", "my-token")?;
/// let course = canvas.get_course(1)?;
/// println!("{}", course.name.unwrap_or_default());
/// # Ok(()) }
/// ```
pub struct CanvasBlocking {
    inner: Canvas,
    rt: Runtime,
}

impl CanvasBlocking {
    /// Create a new blocking Canvas client.
    ///
    /// `base_url` should be the institution root (e.g. `https://canvas.example.edu`),
    /// not including `/api/v1`.
    pub fn new(base_url: &str, access_token: &str) -> Result<Self> {
        let rt = Runtime::new().map_err(CanvasError::Io)?;
        let inner = Canvas::new(base_url, access_token)?;
        Ok(Self { inner, rt })
    }

    /// Create a blocking Canvas client with a custom [`reqwest::Client`].
    pub fn with_client(
        base_url: &str,
        access_token: &str,
        client: reqwest::Client,
    ) -> Result<Self> {
        let rt = Runtime::new().map_err(CanvasError::Io)?;
        let inner = Canvas::with_client(base_url, access_token, client)?;
        Ok(Self { inner, rt })
    }

    // -------------------------------------------------------------------------
    // Courses
    // -------------------------------------------------------------------------

    /// Fetch a single course by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id`
    pub fn get_course(&self, course_id: u64) -> Result<Course> {
        self.rt.block_on(self.inner.get_course(course_id))
    }

    /// Fetch all courses visible to the authenticated user (collects all pages).
    ///
    /// # Canvas API
    /// `GET /api/v1/courses`
    pub fn get_courses(&self) -> Result<Vec<Course>> {
        self.rt.block_on(self.inner.get_courses().collect_all())
    }

    /// Create a new course under an account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:account_id/courses`
    pub fn create_course(&self, account_id: u64, params: CreateCourseParams) -> Result<Course> {
        self.rt
            .block_on(self.inner.create_course(account_id, params))
    }

    /// Delete a course by ID. Canvas returns the deleted course object.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:id`
    pub fn delete_course(&self, course_id: u64) -> Result<Course> {
        self.rt.block_on(self.inner.delete_course(course_id))
    }

    // -------------------------------------------------------------------------
    // Users
    // -------------------------------------------------------------------------

    /// Fetch a single user by ID or `UserId::Current` for the authenticated user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/:id`
    pub fn get_user(&self, user_id: UserId) -> Result<User> {
        self.rt.block_on(self.inner.get_user(user_id))
    }

    /// Fetch the currently authenticated user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/self`
    pub fn get_current_user(&self) -> Result<CurrentUser> {
        self.rt.block_on(self.inner.get_current_user())
    }

    /// Create a new user under an account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:account_id/users`
    pub fn create_user(&self, account_id: u64, params: CreateUserParams) -> Result<User> {
        self.rt
            .block_on(self.inner.create_user(account_id, params))
    }

    // -------------------------------------------------------------------------
    // Accounts
    // -------------------------------------------------------------------------

    /// Fetch a single account by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:id`
    pub fn get_account(&self, account_id: u64) -> Result<Account> {
        self.rt.block_on(self.inner.get_account(account_id))
    }

    /// Fetch all accounts accessible to the authenticated user (collects all pages).
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts`
    pub fn get_accounts(&self) -> Result<Vec<Account>> {
        self.rt.block_on(self.inner.get_accounts().collect_all())
    }
}
