use crate::{
    error::{CanvasError, Result},
    http::Requester,
    pagination::PageStream,
    params::wrap_params,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::resources::params::quiz_params::CreateQuizParams;

/// Parameters for creating a quiz question.
#[derive(Debug, Default, Clone, Serialize)]
pub struct QuizQuestionParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points_possible: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
}

/// Parameters for updating a quiz submission's score and comments.
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateQuizSubmissionParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fudge_points: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub questions: Option<serde_json::Value>,
}

/// A Canvas quiz.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct Quiz {
    pub id: u64,
    pub course_id: Option<u64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub quiz_type: Option<String>,
    pub time_limit: Option<u64>,
    pub shuffle_answers: Option<bool>,
    pub show_correct_answers: Option<bool>,
    pub scoring_policy: Option<String>,
    pub allowed_attempts: Option<i64>,
    pub one_question_at_a_time: Option<bool>,
    pub question_count: Option<u64>,
    pub points_possible: Option<f64>,
    pub due_at: Option<DateTime<Utc>>,
    pub lock_at: Option<DateTime<Utc>>,
    pub unlock_at: Option<DateTime<Utc>>,
    pub published: Option<bool>,
    pub workflow_state: Option<String>,
    pub html_url: Option<String>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Quiz {
    fn course_id_or_err(&self) -> Result<u64> {
        self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "Quiz has no course_id".to_string(),
            errors: vec![],
        })
    }

    fn propagate(&self, q: &mut Quiz) {
        q.requester = self.requester.clone();
        q.course_id = self.course_id;
    }

    fn propagate_sub(&self, s: &mut QuizSubmission) {
        s.requester = self.requester.clone();
        s.course_id = self.course_id;
        s.quiz_id = Some(self.id);
    }

    /// Edit this quiz.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/quizzes/:id`
    pub async fn edit(&self, params: CreateQuizParams) -> Result<Quiz> {
        let course_id = self.course_id_or_err()?;
        let form = wrap_params("quiz", &params);
        let mut q: Quiz = self
            .req()
            .put(&format!("courses/{course_id}/quizzes/{}", self.id), &form)
            .await?;
        self.propagate(&mut q);
        Ok(q)
    }

    /// Delete this quiz.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:course_id/quizzes/:id`
    pub async fn delete(&self) -> Result<Quiz> {
        let course_id = self.course_id_or_err()?;
        let mut q: Quiz = self
            .req()
            .delete(&format!("courses/{course_id}/quizzes/{}", self.id), &[])
            .await?;
        self.propagate(&mut q);
        Ok(q)
    }

    /// Create a question in this quiz.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/quizzes/:id/questions`
    pub async fn create_question(&self, params: QuizQuestionParams) -> Result<QuizQuestion> {
        let course_id = self.course_id_or_err()?;
        let form = wrap_params("question", &params);
        let mut q: QuizQuestion = self
            .req()
            .post(
                &format!("courses/{course_id}/quizzes/{}/questions", self.id),
                &form,
            )
            .await?;
        q.requester = self.requester.clone();
        q.course_id = self.course_id;
        q.quiz_id = Some(self.id);
        Ok(q)
    }

    /// Fetch a single quiz question by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/quizzes/:id/questions/:question_id`
    pub async fn get_question(&self, question_id: u64) -> Result<QuizQuestion> {
        let course_id = self.course_id_or_err()?;
        let mut q: QuizQuestion = self
            .req()
            .get(
                &format!(
                    "courses/{course_id}/quizzes/{}/questions/{question_id}",
                    self.id
                ),
                &[],
            )
            .await?;
        q.requester = self.requester.clone();
        q.course_id = self.course_id;
        q.quiz_id = Some(self.id);
        Ok(q)
    }

    /// Stream all questions in this quiz.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/quizzes/:id/questions`
    pub fn get_questions(&self) -> PageStream<QuizQuestion> {
        let course_id = self.course_id.unwrap_or(0);
        let quiz_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/quizzes/{quiz_id}/questions"),
            vec![],
            move |mut q: QuizQuestion, req| {
                q.requester = Some(Arc::clone(&req));
                q.course_id = Some(course_id);
                q.quiz_id = Some(quiz_id);
                q
            },
        )
    }

    /// Create a new submission (start the quiz).
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/quizzes/:id/submissions`
    pub async fn create_submission(&self) -> Result<QuizSubmission> {
        let course_id = self.course_id_or_err()?;
        let resp: serde_json::Value = self
            .req()
            .post(
                &format!("courses/{course_id}/quizzes/{}/submissions", self.id),
                &[],
            )
            .await?;
        // Canvas wraps in {"quiz_submissions": [...]}
        let sub_val = resp
            .get("quiz_submissions")
            .and_then(|a| a.get(0))
            .cloned()
            .unwrap_or(resp);
        let mut s: QuizSubmission = serde_json::from_value(sub_val)?;
        self.propagate_sub(&mut s);
        Ok(s)
    }

    /// Fetch a single quiz submission by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/quizzes/:id/submissions/:submission_id`
    pub async fn get_submission(&self, submission_id: u64) -> Result<QuizSubmission> {
        let course_id = self.course_id_or_err()?;
        let resp: serde_json::Value = self
            .req()
            .get(
                &format!(
                    "courses/{course_id}/quizzes/{}/submissions/{submission_id}",
                    self.id
                ),
                &[],
            )
            .await?;
        let sub_val = resp
            .get("quiz_submissions")
            .and_then(|a| a.get(0))
            .cloned()
            .unwrap_or(resp);
        let mut s: QuizSubmission = serde_json::from_value(sub_val)?;
        self.propagate_sub(&mut s);
        Ok(s)
    }

    /// Stream all submissions for this quiz.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/quizzes/:id/submissions`
    pub fn get_submissions(&self) -> PageStream<QuizSubmission> {
        let course_id = self.course_id.unwrap_or(0);
        let quiz_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/quizzes/{quiz_id}/submissions"),
            vec![],
            move |mut s: QuizSubmission, req| {
                s.requester = Some(Arc::clone(&req));
                s.course_id = Some(course_id);
                s.quiz_id = Some(quiz_id);
                s
            },
        )
    }

    /// Set extensions for this quiz for one or more students.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/quizzes/:id/extensions`
    pub async fn set_extensions(&self, params: &[(String, String)]) -> Result<serde_json::Value> {
        let course_id = self.course_id_or_err()?;
        self.req()
            .post(
                &format!("courses/{course_id}/quizzes/{}/extensions", self.id),
                params,
            )
            .await
    }

    /// Get statistics for this quiz.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/quizzes/:id/statistics`
    pub async fn get_statistics(&self) -> Result<serde_json::Value> {
        let course_id = self.course_id_or_err()?;
        self.req()
            .get(
                &format!("courses/{course_id}/quizzes/{}/statistics", self.id),
                &[],
            )
            .await
    }
}

/// A question within a Canvas quiz.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct QuizQuestion {
    pub id: u64,
    pub quiz_id: Option<u64>,
    pub question_name: Option<String>,
    pub question_type: Option<String>,
    pub question_text: Option<String>,
    pub points_possible: Option<f64>,
    pub position: Option<u64>,
    pub answers: Option<Vec<serde_json::Value>>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
    #[serde(skip)]
    pub course_id: Option<u64>,
}

/// A student's submission for a Canvas quiz.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct QuizSubmission {
    pub id: u64,
    pub quiz_id: Option<u64>,
    pub user_id: Option<u64>,
    pub attempt: Option<u64>,
    pub workflow_state: Option<String>,
    pub score: Option<f64>,
    pub kept_score: Option<f64>,
    pub fudge_points: Option<f64>,
    pub validation_token: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
    #[serde(skip)]
    pub course_id: Option<u64>,
}

impl QuizSubmission {
    fn course_quiz_prefix(&self) -> Result<String> {
        let course_id = self.course_id.ok_or_else(|| CanvasError::BadRequest {
            message: "QuizSubmission has no course_id".to_string(),
            errors: vec![],
        })?;
        let quiz_id = self.quiz_id.ok_or_else(|| CanvasError::BadRequest {
            message: "QuizSubmission has no quiz_id".to_string(),
            errors: vec![],
        })?;
        Ok(format!(
            "courses/{course_id}/quizzes/{quiz_id}/submissions/{}",
            self.id
        ))
    }

    fn propagate(&self, s: &mut QuizSubmission) {
        s.requester = self.requester.clone();
        s.course_id = self.course_id;
        s.quiz_id = self.quiz_id;
    }

    /// Complete (finish) this quiz submission.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/quizzes/:quiz_id/submissions/:id/complete`
    pub async fn complete(&self, validation_token: &str) -> Result<QuizSubmission> {
        let prefix = self.course_quiz_prefix()?;
        let attempt = self.attempt.unwrap_or(1).to_string();
        let params = vec![
            ("attempt".to_string(), attempt),
            ("validation_token".to_string(), validation_token.to_string()),
        ];
        let resp: serde_json::Value = self
            .req()
            .post(&format!("{prefix}/complete"), &params)
            .await?;
        let sub_val = resp
            .get("quiz_submissions")
            .and_then(|a| a.get(0))
            .cloned()
            .unwrap_or(resp);
        let mut s: QuizSubmission = serde_json::from_value(sub_val)?;
        self.propagate(&mut s);
        Ok(s)
    }

    /// Fetch the questions for this submission.
    ///
    /// # Canvas API
    /// `GET /api/v1/quiz_submissions/:id/questions`
    pub async fn get_submission_questions(&self) -> Result<Vec<serde_json::Value>> {
        let resp: serde_json::Value = self
            .req()
            .get(&format!("quiz_submissions/{}/questions", self.id), &[])
            .await?;
        Ok(resp
            .get("quiz_submission_questions")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default())
    }

    /// Get the time remaining for this submission.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/quizzes/:quiz_id/submissions/:id/time`
    pub async fn get_times(&self) -> Result<serde_json::Value> {
        let prefix = self.course_quiz_prefix()?;
        self.req().get(&format!("{prefix}/time"), &[]).await
    }

    /// Update the score and comments for this submission.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:course_id/quizzes/:quiz_id/submissions/:id`
    pub async fn update_score_and_comments(
        &self,
        params: UpdateQuizSubmissionParams,
    ) -> Result<QuizSubmission> {
        let prefix = self.course_quiz_prefix()?;
        let form = wrap_params("quiz_submissions", &params);
        let resp: serde_json::Value = self.req().put(&prefix, &form).await?;
        let sub_val = resp
            .get("quiz_submissions")
            .and_then(|a| a.get(0))
            .cloned()
            .unwrap_or(resp);
        let mut s: QuizSubmission = serde_json::from_value(sub_val)?;
        self.propagate(&mut s);
        Ok(s)
    }
}
