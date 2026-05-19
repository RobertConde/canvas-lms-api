use crate::{
    error::Result,
    http::Requester,
    pagination::PageStream,
    params::wrap_params,
    resources::{
        assignment::Assignment,
        blueprint::{BlueprintSubscription, BlueprintTemplate},
        content_migration::{ContentMigration, Migrator},
        discussion_topic::DiscussionTopic,
        enrollment::Enrollment,
        external_tool::{ExternalTool, ExternalToolParams},
        file::File,
        gradebook_history::{Day, Grader, SubmissionHistory, SubmissionVersion},
        group::Group,
        module::Module,
        outcome::{OutcomeGroup, OutcomeLink, UpdateOutcomeGroupParams},
        page::Page,
        params::{
            assignment_params::CreateAssignmentParams, course_params::UpdateCourseParams,
            quiz_params::CreateQuizParams,
        },
        quiz::Quiz,
        rubric::{Rubric, RubricAssociation, RubricParams},
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
            .get(&format!("courses/{}/sections/{section_id}", self.id), &[])
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

    /// Upload a file to this course.
    ///
    /// Canvas uses a two-step upload: first POSTing metadata to obtain an upload URL,
    /// then POSTing the file as multipart form data to that URL.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:id/files`
    pub async fn upload_file(
        &self,
        request: crate::upload::UploadRequest,
        data: Vec<u8>,
    ) -> crate::error::Result<crate::resources::file::File> {
        crate::upload::initiate_and_upload(
            self.req(),
            &format!("courses/{}/files", self.id),
            request,
            data,
        )
        .await
    }

    // -------------------------------------------------------------------------
    // External Tools
    // -------------------------------------------------------------------------

    /// Fetch a single external tool by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/external_tools/:id`
    pub async fn get_external_tool(&self, tool_id: u64) -> Result<ExternalTool> {
        let mut tool: ExternalTool = self
            .req()
            .get(
                &format!("courses/{}/external_tools/{tool_id}", self.id),
                &[],
            )
            .await?;
        tool.requester = self.requester.clone();
        Ok(tool)
    }

    /// Stream all external tools for this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/external_tools`
    pub fn get_external_tools(&self) -> PageStream<ExternalTool> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/external_tools"),
            vec![],
            |mut t: ExternalTool, req| {
                t.requester = Some(Arc::clone(&req));
                t
            },
        )
    }

    /// Create an external tool on this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/external_tools`
    pub async fn create_external_tool(&self, params: ExternalToolParams) -> Result<ExternalTool> {
        let form = wrap_params("external_tool", &params);
        let mut tool: ExternalTool = self
            .req()
            .post(&format!("courses/{}/external_tools", self.id), &form)
            .await?;
        tool.requester = self.requester.clone();
        Ok(tool)
    }

    // -------------------------------------------------------------------------
    // Rubrics
    // -------------------------------------------------------------------------

    /// Fetch a single rubric by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/rubrics/:id`
    pub async fn get_rubric(&self, rubric_id: u64) -> Result<Rubric> {
        let mut rubric: Rubric = self
            .req()
            .get(&format!("courses/{}/rubrics/{rubric_id}", self.id), &[])
            .await?;
        rubric.requester = self.requester.clone();
        Ok(rubric)
    }

    /// Stream all rubrics for this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/rubrics`
    pub fn get_rubrics(&self) -> PageStream<Rubric> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/rubrics"),
            vec![],
            |mut r: Rubric, req| {
                r.requester = Some(Arc::clone(&req));
                r
            },
        )
    }

    /// Create a rubric in this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/rubrics`
    pub async fn create_rubric(&self, params: RubricParams) -> Result<Rubric> {
        let form = wrap_params("rubric", &params);
        let mut rubric: Rubric = self
            .req()
            .post(&format!("courses/{}/rubrics", self.id), &form)
            .await?;
        rubric.requester = self.requester.clone();
        Ok(rubric)
    }

    /// Fetch a single rubric association by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/rubric_associations/:id`
    pub async fn get_rubric_association(&self, association_id: u64) -> Result<RubricAssociation> {
        let mut assoc: RubricAssociation = self
            .req()
            .get(
                &format!("courses/{}/rubric_associations/{association_id}", self.id),
                &[],
            )
            .await?;
        assoc.requester = self.requester.clone();
        Ok(assoc)
    }

    /// Stream all rubric associations for this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/rubric_associations`
    pub fn get_rubric_associations(&self) -> PageStream<RubricAssociation> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/rubric_associations"),
            vec![],
            |mut a: RubricAssociation, req| {
                a.requester = Some(Arc::clone(&req));
                a
            },
        )
    }

    // -------------------------------------------------------------------------
    // Blueprint
    // -------------------------------------------------------------------------

    /// Fetch the blueprint template for this course.
    ///
    /// `template_id` is typically `"default"` or a numeric ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/blueprint_templates/:template_id`
    pub async fn get_blueprint(&self, template_id: &str) -> Result<BlueprintTemplate> {
        let mut tmpl: BlueprintTemplate = self
            .req()
            .get(
                &format!("courses/{}/blueprint_templates/{template_id}", self.id),
                &[],
            )
            .await?;
        tmpl.requester = self.requester.clone();
        Ok(tmpl)
    }

    /// Stream blueprint subscriptions for this (child) course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/blueprint_subscriptions`
    pub fn get_blueprint_subscriptions(&self) -> PageStream<BlueprintSubscription> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/blueprint_subscriptions"),
            vec![],
            |mut s: BlueprintSubscription, req| {
                s.requester = Some(Arc::clone(&req));
                s
            },
        )
    }

    // -------------------------------------------------------------------------
    // Content Migrations
    // -------------------------------------------------------------------------

    /// Fetch a single content migration by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/content_migrations/:id`
    pub async fn get_content_migration(&self, migration_id: u64) -> Result<ContentMigration> {
        let mut migration: ContentMigration = self
            .req()
            .get(
                &format!("courses/{}/content_migrations/{migration_id}", self.id),
                &[],
            )
            .await?;
        migration.requester = self.requester.clone();
        Ok(migration)
    }

    /// Stream all content migrations for this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/content_migrations`
    pub fn get_content_migrations(&self) -> PageStream<ContentMigration> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/content_migrations"),
            vec![],
            |mut m: ContentMigration, req| {
                m.requester = Some(Arc::clone(&req));
                m
            },
        )
    }

    /// Create a content migration for this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/content_migrations`
    pub async fn create_content_migration(
        &self,
        migration_type: &str,
        params: &[(String, String)],
    ) -> Result<ContentMigration> {
        let mut form = vec![("migration_type".to_string(), migration_type.to_string())];
        form.extend_from_slice(params);
        let mut migration: ContentMigration = self
            .req()
            .post(&format!("courses/{}/content_migrations", self.id), &form)
            .await?;
        migration.requester = self.requester.clone();
        Ok(migration)
    }

    /// Stream available content migration types for this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/content_migrations/migrators`
    pub fn get_migrators(&self) -> PageStream<Migrator> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/content_migrations/migrators", self.id),
            vec![],
        )
    }

    // -------------------------------------------------------------------------
    // Outcome Groups
    // -------------------------------------------------------------------------

    /// Stream all outcome group links for this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/outcome_group_links`
    pub fn get_outcome_group_links(&self) -> PageStream<OutcomeLink> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/outcome_group_links", self.id),
            vec![],
        )
    }

    /// Fetch a single outcome group by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/outcome_groups/:id`
    pub async fn get_outcome_group(&self, group_id: u64) -> Result<OutcomeGroup> {
        let mut group: OutcomeGroup = self
            .req()
            .get(
                &format!("courses/{}/outcome_groups/{group_id}", self.id),
                &[],
            )
            .await?;
        group.requester = self.requester.clone();
        Ok(group)
    }

    /// Create a top-level outcome group on this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/outcome_groups`
    pub async fn create_outcome_group(
        &self,
        params: UpdateOutcomeGroupParams,
    ) -> Result<OutcomeGroup> {
        let form = wrap_params("outcome_group", &params);
        let mut group: OutcomeGroup = self
            .req()
            .post(&format!("courses/{}/outcome_groups", self.id), &form)
            .await?;
        group.requester = self.requester.clone();
        Ok(group)
    }

    // -------------------------------------------------------------------------
    // Gradebook History
    // -------------------------------------------------------------------------

    /// Stream the days for which there is gradebook history in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/gradebook_history/days`
    pub fn get_gradebook_history_dates(&self) -> PageStream<Day> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/gradebook_history/days", self.id),
            vec![],
        )
    }

    /// Stream graders who worked in this course on a given date.
    ///
    /// `date` should be formatted as `YYYY-MM-DD`.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/gradebook_history/:date`
    pub fn get_gradebook_history_details(&self, date: &str) -> PageStream<Grader> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/gradebook_history/{date}", self.id),
            vec![],
        )
    }

    /// Stream submission versions graded by a specific grader on a specific assignment and date.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/gradebook_history/:date/graders/:grader_id/assignments/:assignment_id/submissions`
    pub fn get_submission_history(
        &self,
        date: &str,
        grader_id: u64,
        assignment_id: u64,
    ) -> PageStream<SubmissionHistory> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!(
                "courses/{}/gradebook_history/{date}/graders/{grader_id}/assignments/{assignment_id}/submissions",
                self.id
            ),
            vec![],
        )
    }

    /// Stream all submission versions (uncollated) for this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/gradebook_history/feed`
    pub fn get_uncollated_submissions(&self) -> PageStream<SubmissionVersion> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/gradebook_history/feed", self.id),
            vec![],
        )
    }

    // -------------------------------------------------------------------------
    // New Quizzes (feature = "new-quizzes")
    // -------------------------------------------------------------------------

    /// Fetch a single New Quiz by its assignment ID.
    ///
    /// # Canvas API
    /// `GET /api/quiz/v1/courses/:course_id/quizzes/:assignment_id`
    #[cfg(feature = "new-quizzes")]
    pub async fn get_new_quiz(
        &self,
        assignment_id: &str,
    ) -> Result<crate::resources::new_quiz::NewQuiz> {
        let mut quiz: crate::resources::new_quiz::NewQuiz = self
            .req()
            .nq_get(&format!("courses/{}/quizzes/{assignment_id}", self.id), &[])
            .await?;
        quiz.requester = self.requester.clone();
        quiz.course_id = Some(self.id);
        Ok(quiz)
    }

    /// Stream all New Quizzes for this course.
    ///
    /// # Canvas API
    /// `GET /api/quiz/v1/courses/:course_id/quizzes`
    #[cfg(feature = "new-quizzes")]
    pub fn get_new_quizzes(&self) -> PageStream<crate::resources::new_quiz::NewQuiz> {
        let course_id = self.id;
        PageStream::new_with_injector_nq(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/quizzes"),
            vec![],
            move |mut q: crate::resources::new_quiz::NewQuiz, req| {
                q.requester = Some(Arc::clone(&req));
                q.course_id = Some(course_id);
                q
            },
        )
    }

    /// Create a New Quiz in this course.
    ///
    /// # Canvas API
    /// `POST /api/quiz/v1/courses/:course_id/quizzes`
    #[cfg(feature = "new-quizzes")]
    pub async fn create_new_quiz(
        &self,
        params: crate::resources::new_quiz::NewQuizParams,
    ) -> Result<crate::resources::new_quiz::NewQuiz> {
        let body = serde_json::to_value(&params).unwrap_or_default();
        let mut quiz: crate::resources::new_quiz::NewQuiz = self
            .req()
            .nq_post(&format!("courses/{}/quizzes", self.id), &body)
            .await?;
        quiz.requester = self.requester.clone();
        quiz.course_id = Some(self.id);
        Ok(quiz)
    }
}
