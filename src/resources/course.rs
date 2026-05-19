use crate::{
    error::Result,
    http::Requester,
    pagination::PageStream,
    params::wrap_params,
    resources::{
        assignment::Assignment,
        discussion_topic::DiscussionTopic,
        enrollment::Enrollment,
        file::File,
        group::Group,
        module::Module,
        page::Page,
        params::{
            assignment_params::CreateAssignmentParams,
            course_params::UpdateCourseParams,
            quiz_params::CreateQuizParams,
        },
        quiz::Quiz,
        section::Section,
        tab::Tab,
        types::WorkflowState,
        user::User,
    },
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Course {
    pub id: u64,
    pub name: Option<String>,
    pub course_code: Option<String>,
    pub workflow_state: Option<WorkflowState>,
    pub account_id: Option<u64>,
    pub root_account_id: Option<u64>,
    pub enrollment_term_id: Option<u64>,
    pub sis_course_id: Option<String>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub grading_standard_id: Option<u64>,
    pub is_public: Option<bool>,
    pub license: Option<String>,
    pub locale: Option<String>,
    pub time_zone: Option<String>,
    pub total_students: Option<u64>,
    pub default_view: Option<String>,
    pub syllabus_body: Option<String>,
    pub public_description: Option<String>,
    pub hide_final_grades: Option<bool>,
    pub apply_assignment_group_weights: Option<bool>,
    pub restrict_enrollments_to_course_dates: Option<bool>,

    #[serde(skip)]
    pub(crate) requester: Option<Arc<Requester>>,
}

impl Course {
    fn req(&self) -> &Arc<Requester> {
        self.requester.as_ref().expect("requester not initialized")
    }

    /// Stream all assignments in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments`
    pub fn get_assignments(&self) -> PageStream<Assignment> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/assignments", self.id),
            vec![],
        )
    }

    /// Fetch a single assignment.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments/:id`
    pub async fn get_assignment(&self, assignment_id: u64) -> Result<Assignment> {
        self.req()
            .get(
                &format!("courses/{}/assignments/{assignment_id}", self.id),
                &[],
            )
            .await
    }

    /// Create a new assignment in this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:id/assignments`
    pub async fn create_assignment(&self, params: CreateAssignmentParams) -> Result<Assignment> {
        let form = wrap_params("assignment", &params);
        self.req()
            .post(&format!("courses/{}/assignments", self.id), &form)
            .await
    }

    /// Stream all sections in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/sections`
    pub fn get_sections(&self) -> PageStream<Section> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/sections", self.id),
            vec![],
        )
    }

    /// Fetch a single section by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/sections/:section_id`
    pub async fn get_section(&self, section_id: u64) -> Result<Section> {
        self.req()
            .get(
                &format!("courses/{}/sections/{section_id}", self.id),
                &[],
            )
            .await
    }

    /// Stream all enrollments in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/enrollments`
    pub fn get_enrollments(&self) -> PageStream<Enrollment> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/enrollments", self.id),
            vec![],
        )
    }

    /// Stream all users in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/users`
    pub fn get_users(&self) -> PageStream<User> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/users", self.id),
            vec![],
        )
    }

    /// Update this course.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:id`
    pub async fn update(&self, params: UpdateCourseParams) -> Result<Course> {
        let form = wrap_params("course", &params);
        let mut course: Course = self
            .req()
            .put(&format!("courses/{}", self.id), &form)
            .await?;
        course.requester = self.requester.clone();
        Ok(course)
    }

    /// Delete this course. Canvas returns the deleted course object.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:id`
    pub async fn delete(&self) -> Result<Course> {
        let params = vec![("event".to_string(), "delete".to_string())];
        let mut course: Course = self
            .req()
            .delete(&format!("courses/{}", self.id), &params)
            .await?;
        course.requester = self.requester.clone();
        Ok(course)
    }

    /// Stream all quizzes in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/quizzes`
    pub fn get_quizzes(&self) -> PageStream<Quiz> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/quizzes", self.id),
            vec![],
        )
    }

    /// Fetch a single quiz.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/quizzes/:quiz_id`
    pub async fn get_quiz(&self, quiz_id: u64) -> Result<Quiz> {
        self.req()
            .get(&format!("courses/{}/quizzes/{quiz_id}", self.id), &[])
            .await
    }

    /// Create a new quiz in this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:id/quizzes`
    pub async fn create_quiz(&self, params: CreateQuizParams) -> Result<Quiz> {
        let form = wrap_params("quiz", &params);
        self.req()
            .post(&format!("courses/{}/quizzes", self.id), &form)
            .await
    }

    /// Stream all modules in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/modules`
    pub fn get_modules(&self) -> PageStream<Module> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/modules", self.id),
            vec![],
        )
    }

    /// Fetch a single module.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/modules/:module_id`
    pub async fn get_module(&self, module_id: u64) -> Result<Module> {
        self.req()
            .get(&format!("courses/{}/modules/{module_id}", self.id), &[])
            .await
    }

    /// Stream all pages in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/pages`
    pub fn get_pages(&self) -> PageStream<Page> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/pages", self.id),
            vec![],
        )
    }

    /// Fetch a single page by URL slug or ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/pages/:url_or_id`
    pub async fn get_page(&self, url_or_id: &str) -> Result<Page> {
        self.req()
            .get(&format!("courses/{}/pages/{url_or_id}", self.id), &[])
            .await
    }

    /// Stream all discussion topics in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/discussion_topics`
    pub fn get_discussion_topics(&self) -> PageStream<DiscussionTopic> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/discussion_topics", self.id),
            vec![],
        )
    }

    /// Fetch a single discussion topic.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/discussion_topics/:topic_id`
    pub async fn get_discussion_topic(&self, topic_id: u64) -> Result<DiscussionTopic> {
        self.req()
            .get(
                &format!("courses/{}/discussion_topics/{topic_id}", self.id),
                &[],
            )
            .await
    }

    /// Stream all files in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/files`
    pub fn get_files(&self) -> PageStream<File> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/files", self.id),
            vec![],
        )
    }

    /// Stream all tabs in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/tabs`
    pub fn get_tabs(&self) -> PageStream<Tab> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/tabs", self.id),
            vec![],
        )
    }

    /// Stream all groups in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/groups`
    pub fn get_groups(&self) -> PageStream<Group> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/groups", self.id),
            vec![],
        )
    }
}
