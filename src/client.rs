use crate::{
    error::Result,
    http::Requester,
    pagination::PageStream,
    params::wrap_params,
    resources::{
        account::Account,
        appointment_group::{AppointmentGroup, AppointmentGroupParams},
        calendar_event::{CalendarEvent, CalendarEventParams},
        conversation::{Conversation, ConversationParams},
        course::Course,
        eportfolio::EPortfolio,
        file::File,
        folder::Folder,
        group::Group,
        jwt::CanvasJwt,
        outcome::Outcome,
        params::{course_params::CreateCourseParams, user_params::CreateUserParams},
        planner::{PlannerNote, PlannerNoteParams, PlannerOverride},
        poll::{CreatePollParams, Poll},
        progress::Progress,
        section::Section,
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
        PageStream::new_with_injector(
            Arc::clone(&self.requester),
            "courses",
            vec![],
            |mut c: Course, req| {
                c.requester = Some(Arc::clone(&req));
                c
            },
        )
    }

    /// Create a new course under an account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:account_id/courses`
    pub async fn create_course(
        &self,
        account_id: u64,
        params: CreateCourseParams,
    ) -> Result<Course> {
        let form = wrap_params("course", &params);
        let mut course: Course = self
            .requester
            .post(&format!("accounts/{account_id}/courses"), &form)
            .await?;
        course.requester = Some(Arc::clone(&self.requester));
        Ok(course)
    }

    /// Delete a course by ID. Canvas returns the deleted course object.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:id`
    pub async fn delete_course(&self, course_id: u64) -> Result<Course> {
        let params = vec![("event".to_string(), "delete".to_string())];
        let mut course: Course = self
            .requester
            .delete(&format!("courses/{course_id}"), &params)
            .await?;
        course.requester = Some(Arc::clone(&self.requester));
        Ok(course)
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
        let mut user: User = self.requester.get(&format!("users/{id}"), &[]).await?;
        user.requester = Some(Arc::clone(&self.requester));
        Ok(user)
    }

    /// Fetch the currently authenticated user.
    ///
    /// # Canvas API
    /// `GET /api/v1/users/self`
    pub async fn get_current_user(&self) -> Result<CurrentUser> {
        self.requester.get("users/self", &[]).await
    }

    /// Create a new user under an account.
    ///
    /// # Canvas API
    /// `POST /api/v1/accounts/:account_id/users`
    pub async fn create_user(&self, account_id: u64, params: CreateUserParams) -> Result<User> {
        let form = wrap_params("user", &params);
        let mut user: User = self
            .requester
            .post(&format!("accounts/{account_id}/users"), &form)
            .await?;
        user.requester = Some(Arc::clone(&self.requester));
        Ok(user)
    }

    // -------------------------------------------------------------------------
    // Accounts
    // -------------------------------------------------------------------------

    /// Fetch a single account by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts/:id`
    pub async fn get_account(&self, account_id: u64) -> Result<Account> {
        let mut account: Account = self
            .requester
            .get(&format!("accounts/{account_id}"), &[])
            .await?;
        account.requester = Some(Arc::clone(&self.requester));
        Ok(account)
    }

    /// Fetch a single outcome by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/outcomes/:id`
    pub async fn get_outcome(&self, outcome_id: u64) -> Result<Outcome> {
        let mut outcome: Outcome = self
            .requester
            .get(&format!("outcomes/{outcome_id}"), &[])
            .await?;
        outcome.requester = Some(Arc::clone(&self.requester));
        Ok(outcome)
    }

    /// Stream all accounts accessible to the authenticated user.
    ///
    /// # Canvas API
    /// `GET /api/v1/accounts`
    pub fn get_accounts(&self) -> PageStream<Account> {
        PageStream::new_with_injector(
            Arc::clone(&self.requester),
            "accounts",
            vec![],
            |mut a: Account, req| {
                a.requester = Some(Arc::clone(&req));
                a
            },
        )
    }

    // -------------------------------------------------------------------------
    // Convenience accessors for existing resources
    // -------------------------------------------------------------------------

    /// Fetch a single section by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/sections/:id`
    pub async fn get_section(&self, section_id: u64) -> Result<Section> {
        self.requester
            .get(&format!("sections/{section_id}"), &[])
            .await
    }

    /// Fetch a single group by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/groups/:id`
    pub async fn get_group(&self, group_id: u64) -> Result<Group> {
        let mut g: Group = self
            .requester
            .get(&format!("groups/{group_id}"), &[])
            .await?;
        g.requester = Some(Arc::clone(&self.requester));
        Ok(g)
    }

    /// Fetch a single file by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/files/:id`
    pub async fn get_file(&self, file_id: u64) -> Result<File> {
        self.requester.get(&format!("files/{file_id}"), &[]).await
    }

    /// Fetch a single folder by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/folders/:id`
    pub async fn get_folder(&self, folder_id: u64) -> Result<Folder> {
        self.requester
            .get(&format!("folders/{folder_id}"), &[])
            .await
    }

    /// Fetch a single progress object by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/progress/:id`
    pub async fn get_progress(&self, progress_id: u64) -> Result<Progress> {
        self.requester
            .get(&format!("progress/{progress_id}"), &[])
            .await
    }

    // -------------------------------------------------------------------------
    // Conversations
    // -------------------------------------------------------------------------

    /// Fetch a single conversation by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/conversations/:id`
    pub async fn get_conversation(&self, conversation_id: u64) -> Result<Conversation> {
        let mut c: Conversation = self
            .requester
            .get(&format!("conversations/{conversation_id}"), &[])
            .await?;
        c.requester = Some(Arc::clone(&self.requester));
        Ok(c)
    }

    /// Stream all conversations for the authenticated user.
    ///
    /// # Canvas API
    /// `GET /api/v1/conversations`
    pub fn get_conversations(&self) -> PageStream<Conversation> {
        PageStream::new_with_injector(
            Arc::clone(&self.requester),
            "conversations",
            vec![],
            |mut c: Conversation, req| {
                c.requester = Some(Arc::clone(&req));
                c
            },
        )
    }

    /// Create a new conversation.
    ///
    /// # Canvas API
    /// `POST /api/v1/conversations`
    pub async fn create_conversation(
        &self,
        recipients: &[&str],
        body: &str,
        params: ConversationParams,
    ) -> Result<Conversation> {
        let mut form: Vec<(String, String)> = recipients
            .iter()
            .map(|r| ("recipients[]".into(), r.to_string()))
            .collect();
        form.push(("body".into(), body.to_string()));
        if let Some(subject) = params.subject {
            form.push(("subject".into(), subject));
        }
        if let Some(fg) = params.force_new {
            form.push(("force_new".into(), fg.to_string()));
        }
        if let Some(gc) = params.group_conversation {
            form.push(("group_conversation".into(), gc.to_string()));
        }
        if let Some(ctx) = params.context_code {
            form.push(("context_code".into(), ctx));
        }
        // Canvas returns an array; take the first element
        let result: serde_json::Value = self.requester.post("conversations", &form).await?;
        let first = result
            .as_array()
            .and_then(|a| a.first())
            .cloned()
            .unwrap_or(result);
        let mut c: Conversation = serde_json::from_value(first)?;
        c.requester = Some(Arc::clone(&self.requester));
        Ok(c)
    }

    // -------------------------------------------------------------------------
    // Calendar Events
    // -------------------------------------------------------------------------

    /// Fetch a single calendar event by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/calendar_events/:id`
    pub async fn get_calendar_event(&self, event_id: u64) -> Result<CalendarEvent> {
        let mut e: CalendarEvent = self
            .requester
            .get(&format!("calendar_events/{event_id}"), &[])
            .await?;
        e.requester = Some(Arc::clone(&self.requester));
        Ok(e)
    }

    /// Stream all calendar events visible to the authenticated user.
    ///
    /// # Canvas API
    /// `GET /api/v1/calendar_events`
    pub fn get_calendar_events(&self) -> PageStream<CalendarEvent> {
        PageStream::new_with_injector(
            Arc::clone(&self.requester),
            "calendar_events",
            vec![],
            |mut e: CalendarEvent, req| {
                e.requester = Some(Arc::clone(&req));
                e
            },
        )
    }

    /// Create a new calendar event.
    ///
    /// # Canvas API
    /// `POST /api/v1/calendar_events`
    pub async fn create_calendar_event(
        &self,
        context_code: &str,
        params: CalendarEventParams,
    ) -> Result<CalendarEvent> {
        let body = serde_json::to_value(&params).unwrap_or_default();
        let mut form = wrap_params("calendar_event", &body);
        form.push((
            "calendar_event[context_code]".into(),
            context_code.to_string(),
        ));
        let mut e: CalendarEvent = self.requester.post("calendar_events", &form).await?;
        e.requester = Some(Arc::clone(&self.requester));
        Ok(e)
    }

    // -------------------------------------------------------------------------
    // Planner
    // -------------------------------------------------------------------------

    /// Fetch a single planner note by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/planner_notes/:id`
    pub async fn get_planner_note(&self, note_id: u64) -> Result<PlannerNote> {
        let mut n: PlannerNote = self
            .requester
            .get(&format!("planner_notes/{note_id}"), &[])
            .await?;
        n.requester = Some(Arc::clone(&self.requester));
        Ok(n)
    }

    /// Stream all planner notes for the authenticated user.
    ///
    /// # Canvas API
    /// `GET /api/v1/planner_notes`
    pub fn get_planner_notes(&self) -> PageStream<PlannerNote> {
        PageStream::new_with_injector(
            Arc::clone(&self.requester),
            "planner_notes",
            vec![],
            |mut n: PlannerNote, req| {
                n.requester = Some(Arc::clone(&req));
                n
            },
        )
    }

    /// Create a planner note for the authenticated user.
    ///
    /// # Canvas API
    /// `POST /api/v1/planner_notes`
    pub async fn create_planner_note(&self, params: PlannerNoteParams) -> Result<PlannerNote> {
        let flat: Vec<(String, String)> = serde_json::to_value(&params)
            .unwrap_or_default()
            .as_object()
            .into_iter()
            .flatten()
            .filter_map(|(k, v)| {
                v.as_str()
                    .map(|s| (k.clone(), s.to_string()))
                    .or_else(|| v.as_u64().map(|n| (k.clone(), n.to_string())))
            })
            .collect();
        let mut n: PlannerNote = self.requester.post("planner_notes", &flat).await?;
        n.requester = Some(Arc::clone(&self.requester));
        Ok(n)
    }

    /// Fetch a single planner override by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/planner/overrides/:id`
    pub async fn get_planner_override(&self, override_id: u64) -> Result<PlannerOverride> {
        let mut o: PlannerOverride = self
            .requester
            .get(&format!("planner/overrides/{override_id}"), &[])
            .await?;
        o.requester = Some(Arc::clone(&self.requester));
        Ok(o)
    }

    /// Stream all planner overrides for the authenticated user.
    ///
    /// # Canvas API
    /// `GET /api/v1/planner/overrides`
    pub fn get_planner_overrides(&self) -> PageStream<PlannerOverride> {
        PageStream::new_with_injector(
            Arc::clone(&self.requester),
            "planner/overrides",
            vec![],
            |mut o: PlannerOverride, req| {
                o.requester = Some(Arc::clone(&req));
                o
            },
        )
    }

    /// Create a planner override for a specific plannable item.
    ///
    /// # Canvas API
    /// `POST /api/v1/planner/overrides`
    pub async fn create_planner_override(
        &self,
        plannable_type: &str,
        plannable_id: u64,
    ) -> Result<PlannerOverride> {
        let form = vec![
            ("plannable_type".into(), plannable_type.to_string()),
            ("plannable_id".into(), plannable_id.to_string()),
        ];
        let mut o: PlannerOverride = self.requester.post("planner/overrides", &form).await?;
        o.requester = Some(Arc::clone(&self.requester));
        Ok(o)
    }

    // -------------------------------------------------------------------------
    // ePortfolios
    // -------------------------------------------------------------------------

    /// Fetch a single ePortfolio by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/eportfolios/:id`
    pub async fn get_eportfolio(&self, eportfolio_id: u64) -> Result<EPortfolio> {
        let mut p: EPortfolio = self
            .requester
            .get(&format!("eportfolios/{eportfolio_id}"), &[])
            .await?;
        p.requester = Some(Arc::clone(&self.requester));
        Ok(p)
    }

    // -------------------------------------------------------------------------
    // Appointment Groups
    // -------------------------------------------------------------------------

    /// Fetch a single appointment group by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/appointment_groups/:id`
    pub async fn get_appointment_group(&self, group_id: u64) -> Result<AppointmentGroup> {
        let mut a: AppointmentGroup = self
            .requester
            .get(&format!("appointment_groups/{group_id}"), &[])
            .await?;
        a.requester = Some(Arc::clone(&self.requester));
        Ok(a)
    }

    /// Stream all appointment groups visible to the authenticated user.
    ///
    /// # Canvas API
    /// `GET /api/v1/appointment_groups`
    pub fn get_appointment_groups(&self) -> PageStream<AppointmentGroup> {
        PageStream::new_with_injector(
            Arc::clone(&self.requester),
            "appointment_groups",
            vec![],
            |mut a: AppointmentGroup, req| {
                a.requester = Some(Arc::clone(&req));
                a
            },
        )
    }

    /// Create a new appointment group.
    ///
    /// # Canvas API
    /// `POST /api/v1/appointment_groups`
    pub async fn create_appointment_group(
        &self,
        params: AppointmentGroupParams,
    ) -> Result<AppointmentGroup> {
        let body = serde_json::to_value(&params).unwrap_or_default();
        let form = wrap_params("appointment_group", &body);
        let mut a: AppointmentGroup = self.requester.post("appointment_groups", &form).await?;
        a.requester = Some(Arc::clone(&self.requester));
        Ok(a)
    }

    // -------------------------------------------------------------------------
    // GraphQL (feature = "graphql")
    // -------------------------------------------------------------------------

    /// Return a [`GraphQL`][crate::graphql::GraphQL] client for this Canvas instance.
    ///
    /// # Example
    /// ```no_run
    /// # #[tokio::main] async fn main() -> canvas_lms_api::Result<()> {
    /// let canvas = canvas_lms_api::Canvas::new("https://canvas.example.edu", "token")?;
    /// let gql = canvas.graphql();
    /// let res = gql.query("{ allCourses { id name } }", None).await?;
    /// # Ok(()) }
    /// ```
    #[cfg(feature = "graphql")]
    pub fn graphql(&self) -> crate::graphql::GraphQL {
        crate::graphql::GraphQL {
            requester: Arc::clone(&self.requester),
        }
    }

    // -------------------------------------------------------------------------
    // JWT
    // -------------------------------------------------------------------------

    // -------------------------------------------------------------------------
    // Polls
    // -------------------------------------------------------------------------

    /// Fetch a single poll by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/polls/:id`
    pub async fn get_poll(&self, poll_id: u64) -> Result<Poll> {
        let val: serde_json::Value = self.requester.get(&format!("polls/{poll_id}"), &[]).await?;
        let mut poll: Poll = serde_json::from_value(val["polls"][0].clone())?;
        poll.requester = Some(Arc::clone(&self.requester));
        Ok(poll)
    }

    /// Stream all polls for the current user.
    ///
    /// # Canvas API
    /// `GET /api/v1/polls`
    pub fn get_polls(&self) -> PageStream<Poll> {
        PageStream::new_with_injector(
            Arc::clone(&self.requester),
            "polls",
            vec![],
            |mut p: Poll, req| {
                p.requester = Some(Arc::clone(&req));
                p
            },
        )
    }

    /// Create a new poll for the current user.
    ///
    /// # Canvas API
    /// `POST /api/v1/polls`
    pub async fn create_poll(&self, params: CreatePollParams) -> Result<Poll> {
        let form = wrap_params("polls[]", &params);
        let val: serde_json::Value = self.requester.post("polls", &form).await?;
        let mut poll: Poll = serde_json::from_value(val["polls"][0].clone())?;
        poll.requester = Some(Arc::clone(&self.requester));
        Ok(poll)
    }

    // -------------------------------------------------------------------------

    /// Create a short-lived JWT for use with other Canvas services.
    ///
    /// # Canvas API
    /// `POST /api/v1/jwts`
    pub async fn create_jwt(&self) -> Result<CanvasJwt> {
        self.requester.post("jwts", &[]).await
    }

    /// Refresh an existing JWT, returning a new one.
    ///
    /// # Canvas API
    /// `POST /api/v1/jwts/refresh`
    pub async fn refresh_jwt(&self, token: &str) -> Result<CanvasJwt> {
        let params = vec![("jwt".into(), token.to_string())];
        self.requester.post("jwts/refresh", &params).await
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
