use crate::{error::Result, http::Requester, pagination::PageStream, params::wrap_params};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Param structs
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone, Serialize)]
pub struct CreatePollParams {
    pub question: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct PollChoiceParams {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_correct: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u32>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct PollSessionParams {
    pub course_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub course_section_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_public_results: Option<bool>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct PollSubmissionParams {
    pub poll_choice_id: u64,
}

// ---------------------------------------------------------------------------
// Poll
// ---------------------------------------------------------------------------

/// A Canvas poll created by the current user.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct Poll {
    pub id: u64,
    pub question: Option<String>,
    pub description: Option<String>,
    pub total_results: Option<serde_json::Value>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Poll {
    fn inject(&mut self, req: &Arc<Requester>) {
        self.requester = Some(Arc::clone(req));
    }

    // -------------------------------------------------------------------------
    // Poll-level
    // -------------------------------------------------------------------------

    /// Update this poll.
    ///
    /// # Canvas API
    /// `PUT /api/v1/polls/:id`
    pub async fn update(&self, params: CreatePollParams) -> Result<Poll> {
        let form = wrap_params("polls[]", &params);
        let val: serde_json::Value = self.req().put(&format!("polls/{}", self.id), &form).await?;
        let mut poll: Poll = serde_json::from_value(val["polls"][0].clone())?;
        poll.inject(self.req());
        Ok(poll)
    }

    /// Delete this poll.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/polls/:id`
    pub async fn delete(&self) -> Result<()> {
        self.req().delete_void(&format!("polls/{}", self.id)).await
    }

    // -------------------------------------------------------------------------
    // Poll choices
    // -------------------------------------------------------------------------

    /// Fetch a single poll choice.
    ///
    /// # Canvas API
    /// `GET /api/v1/polls/:poll_id/poll_choices/:id`
    pub async fn get_choice(&self, choice_id: u64) -> Result<PollChoice> {
        let val: serde_json::Value = self
            .req()
            .get(
                &format!("polls/{}/poll_choices/{}", self.id, choice_id),
                &[],
            )
            .await?;
        let mut choice: PollChoice = serde_json::from_value(val["poll_choices"][0].clone())?;
        choice.requester = self.requester.clone();
        Ok(choice)
    }

    /// Stream all choices for this poll.
    ///
    /// # Canvas API
    /// `GET /api/v1/polls/:poll_id/poll_choices`
    pub fn get_choices(&self) -> PageStream<PollChoice> {
        let poll_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("polls/{poll_id}/poll_choices"),
            vec![],
            |mut c: PollChoice, req| {
                c.requester = Some(Arc::clone(&req));
                c
            },
        )
    }

    /// Create a new choice for this poll.
    ///
    /// # Canvas API
    /// `POST /api/v1/polls/:poll_id/poll_choices`
    pub async fn create_choice(&self, params: PollChoiceParams) -> Result<PollChoice> {
        let form = wrap_params("poll_choice[]", &params);
        let val: serde_json::Value = self
            .req()
            .post(&format!("polls/{}/poll_choices", self.id), &form)
            .await?;
        let mut choice: PollChoice = serde_json::from_value(val["poll_choices"][0].clone())?;
        choice.requester = self.requester.clone();
        Ok(choice)
    }

    // -------------------------------------------------------------------------
    // Poll sessions
    // -------------------------------------------------------------------------

    /// Fetch a single poll session.
    ///
    /// # Canvas API
    /// `GET /api/v1/polls/:poll_id/poll_sessions/:id`
    pub async fn get_session(&self, session_id: u64) -> Result<PollSession> {
        let val: serde_json::Value = self
            .req()
            .get(
                &format!("polls/{}/poll_sessions/{}", self.id, session_id),
                &[],
            )
            .await?;
        let mut session: PollSession = serde_json::from_value(val["poll_sessions"][0].clone())?;
        session.requester = self.requester.clone();
        Ok(session)
    }

    /// Stream all sessions for this poll.
    ///
    /// # Canvas API
    /// `GET /api/v1/polls/:poll_id/poll_sessions`
    pub fn get_sessions(&self) -> PageStream<PollSession> {
        let poll_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("polls/{poll_id}/poll_sessions"),
            vec![],
            |mut s: PollSession, req| {
                s.requester = Some(Arc::clone(&req));
                s
            },
        )
    }

    /// Create a new session for this poll.
    ///
    /// # Canvas API
    /// `POST /api/v1/polls/:poll_id/poll_sessions`
    pub async fn create_session(&self, params: PollSessionParams) -> Result<PollSession> {
        let form = wrap_params("poll_session[]", &params);
        let val: serde_json::Value = self
            .req()
            .post(&format!("polls/{}/poll_sessions", self.id), &form)
            .await?;
        let mut session: PollSession = serde_json::from_value(val["poll_sessions"][0].clone())?;
        session.requester = self.requester.clone();
        Ok(session)
    }
}

// ---------------------------------------------------------------------------
// PollChoice
// ---------------------------------------------------------------------------

/// A choice within a Canvas poll.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PollChoice {
    pub id: u64,
    pub poll_id: Option<u64>,
    pub text: Option<String>,
    pub is_correct: Option<bool>,
    pub position: Option<u32>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl PollChoice {
    /// Update this poll choice.
    ///
    /// # Canvas API
    /// `PUT /api/v1/polls/:poll_id/poll_choices/:id`
    pub async fn update(&self, params: PollChoiceParams) -> Result<PollChoice> {
        let poll_id = self.poll_id.unwrap_or_default();
        let form = wrap_params("poll_choice[]", &params);
        let val: serde_json::Value = self
            .req()
            .put(&format!("polls/{poll_id}/poll_choices/{}", self.id), &form)
            .await?;
        let mut choice: PollChoice = serde_json::from_value(val["poll_choices"][0].clone())?;
        choice.requester = self.requester.clone();
        Ok(choice)
    }

    /// Delete this poll choice.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/polls/:poll_id/poll_choices/:id`
    pub async fn delete(&self) -> Result<()> {
        let poll_id = self.poll_id.unwrap_or_default();
        self.req()
            .delete_void(&format!("polls/{poll_id}/poll_choices/{}", self.id))
            .await
    }

    fn req(&self) -> &Arc<Requester> {
        self.requester.as_ref().expect("requester not initialized")
    }
}

// ---------------------------------------------------------------------------
// PollSession
// ---------------------------------------------------------------------------

/// A session of a Canvas poll (tied to a specific course/section).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PollSession {
    pub id: u64,
    pub poll_id: Option<u64>,
    pub course_id: Option<u64>,
    pub course_section_id: Option<u64>,
    pub is_published: Option<bool>,
    pub has_public_results: Option<bool>,
    pub results: Option<serde_json::Value>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl PollSession {
    fn req(&self) -> &Arc<Requester> {
        self.requester.as_ref().expect("requester not initialized")
    }

    fn poll_id(&self) -> u64 {
        self.poll_id.unwrap_or_default()
    }

    fn endpoint(&self) -> String {
        format!("polls/{}/poll_sessions/{}", self.poll_id(), self.id)
    }

    /// Update this poll session.
    ///
    /// # Canvas API
    /// `PUT /api/v1/polls/:poll_id/poll_sessions/:id`
    pub async fn update(&self, params: PollSessionParams) -> Result<PollSession> {
        let form = wrap_params("poll_session[]", &params);
        let val: serde_json::Value = self.req().put(&self.endpoint(), &form).await?;
        let mut session: PollSession = serde_json::from_value(val["poll_sessions"][0].clone())?;
        session.requester = self.requester.clone();
        Ok(session)
    }

    /// Delete this poll session.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/polls/:poll_id/poll_sessions/:id`
    pub async fn delete(&self) -> Result<()> {
        self.req().delete_void(&self.endpoint()).await
    }

    /// Open this poll session for responses.
    ///
    /// # Canvas API
    /// `GET /api/v1/polls/:poll_id/poll_sessions/:id/open`
    pub async fn open(&self) -> Result<PollSession> {
        let val: serde_json::Value = self
            .req()
            .get(&format!("{}/open", self.endpoint()), &[])
            .await?;
        let mut session: PollSession = serde_json::from_value(val["poll_sessions"][0].clone())?;
        session.requester = self.requester.clone();
        Ok(session)
    }

    /// Close this poll session to responses.
    ///
    /// # Canvas API
    /// `GET /api/v1/polls/:poll_id/poll_sessions/:id/close`
    pub async fn close(&self) -> Result<PollSession> {
        let val: serde_json::Value = self
            .req()
            .get(&format!("{}/close", self.endpoint()), &[])
            .await?;
        let mut session: PollSession = serde_json::from_value(val["poll_sessions"][0].clone())?;
        session.requester = self.requester.clone();
        Ok(session)
    }

    /// Fetch a single poll submission.
    ///
    /// # Canvas API
    /// `GET /api/v1/polls/:poll_id/poll_sessions/:poll_session_id/poll_submissions/:id`
    pub async fn get_submission(&self, submission_id: u64) -> Result<PollSubmission> {
        let val: serde_json::Value = self
            .req()
            .get(
                &format!("{}/poll_submissions/{submission_id}", self.endpoint()),
                &[],
            )
            .await?;
        Ok(serde_json::from_value(val["poll_submissions"][0].clone())?)
    }

    /// Create a poll submission for this session.
    ///
    /// # Canvas API
    /// `POST /api/v1/polls/:poll_id/poll_sessions/:poll_session_id/poll_submissions`
    pub async fn create_submission(&self, params: PollSubmissionParams) -> Result<PollSubmission> {
        let form = wrap_params("poll_submissions[]", &params);
        let val: serde_json::Value = self
            .req()
            .post(&format!("{}/poll_submissions", self.endpoint()), &form)
            .await?;
        Ok(serde_json::from_value(val["poll_submissions"][0].clone())?)
    }
}

// ---------------------------------------------------------------------------
// PollSubmission
// ---------------------------------------------------------------------------

/// A student's response to a poll session.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PollSubmission {
    pub id: u64,
    pub poll_session_id: Option<u64>,
    pub poll_choice_id: Option<u64>,
    pub user_id: Option<u64>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
