use crate::{error::Result, http::Requester};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// A Canvas New Quiz (created via the New Quizzes engine).
///
/// Requires the `new-quizzes` feature. All methods hit `/api/quiz/v1/`
/// rather than the standard `/api/v1/` base.
#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
pub struct NewQuiz {
    pub id: Option<String>,
    pub course_id: Option<u64>,
    pub title: Option<String>,
    pub assignment_id: Option<String>,
    pub points_possible: Option<f64>,
    pub due_at: Option<String>,
    pub lock_at: Option<String>,
    pub unlock_at: Option<String>,
    pub shuffle_answers: Option<bool>,
    pub shuffle_questions: Option<bool>,
    pub require_lockdown_browser: Option<bool>,
    pub show_correct_answers: Option<bool>,
    pub instructions: Option<String>,
    pub workflow_state: Option<String>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

/// JSON body for creating or updating a New Quiz.
#[derive(Debug, Default, Clone, Serialize)]
pub struct NewQuizParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points_possible: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlock_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shuffle_answers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shuffle_questions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

impl NewQuiz {
    fn course_id(&self) -> u64 {
        self.course_id.expect("NewQuiz missing course_id")
    }

    fn quiz_id(&self) -> &str {
        self.id.as_deref().expect("NewQuiz missing id")
    }

    fn endpoint(&self) -> String {
        format!("courses/{}/quizzes/{}", self.course_id(), self.quiz_id())
    }

    /// Update this quiz.
    ///
    /// # Canvas API
    /// `PATCH /api/quiz/v1/courses/:course_id/quizzes/:id`
    pub async fn update(&self, params: NewQuizParams) -> Result<NewQuiz> {
        let body = serde_json::to_value(&params).unwrap_or_default();
        let mut quiz: NewQuiz = self.req().nq_patch(&self.endpoint(), &body).await?;
        quiz.requester = self.requester.clone();
        Ok(quiz)
    }

    /// Delete this quiz.
    ///
    /// # Canvas API
    /// `DELETE /api/quiz/v1/courses/:course_id/quizzes/:id`
    pub async fn delete(&self) -> Result<NewQuiz> {
        let mut quiz: NewQuiz = self.req().nq_delete(&self.endpoint()).await?;
        quiz.requester = self.requester.clone();
        Ok(quiz)
    }

    /// Apply quiz-level accommodations for specific students.
    ///
    /// `accommodations` should be a JSON array of objects with `user_id` and
    /// optional `extra_time`, `extra_attempts`, `reduce_choices_enabled` fields.
    ///
    /// # Canvas API
    /// `POST /api/quiz/v1/courses/:course_id/quizzes/:id/accommodations`
    pub async fn set_accommodations(&self, accommodations: Value) -> Result<Value> {
        self.req()
            .nq_post(
                &format!("{}/accommodations", self.endpoint()),
                &accommodations,
            )
            .await
    }
}
