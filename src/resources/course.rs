use crate::{
    error::Result,
    http::Requester,
    pagination::PageStream,
    params::wrap_params,
    resources::{
        assignment::{Assignment, AssignmentGroup},
        blueprint::{BlueprintSubscription, BlueprintTemplate},
        collaboration::Collaboration,
        content_export::{ContentExport, ContentExportParams},
        content_migration::{ContentMigration, Migrator},
        custom_gradebook_column::{CustomGradebookColumn, CustomGradebookColumnParams},
        discussion_topic::DiscussionTopic,
        enrollment::Enrollment,
        external_tool::{ExternalTool, ExternalToolParams},
        feature::{Feature, FeatureFlag},
        file::File,
        grade_change_log::GradeChangeEvent,
        gradebook_history::{Day, Grader, SubmissionHistory, SubmissionVersion},
        grading_period::GradingPeriod,
        grading_standard::GradingStandard,
        group::{Group, GroupCategory},
        lti_resource_link::{CreateLtiResourceLinkParams, LtiResourceLink},
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

#[derive(Debug, Clone, Deserialize, Serialize, canvas_lms_api_derive::CanvasResource)]
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
    /// Stream all assignments in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments`
    pub fn get_assignments(&self) -> PageStream<Assignment> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{}/assignments", self.id),
            vec![],
            move |mut a: Assignment, req| {
                a.requester = Some(Arc::clone(&req));
                a.course_id = Some(course_id);
                a
            },
        )
    }

    /// Fetch a single assignment.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/assignments/:id`
    pub async fn get_assignment(&self, assignment_id: u64) -> Result<Assignment> {
        let mut a: Assignment = self
            .req()
            .get(
                &format!("courses/{}/assignments/{assignment_id}", self.id),
                &[],
            )
            .await?;
        a.requester = Some(Arc::clone(self.req()));
        a.course_id = Some(self.id);
        Ok(a)
    }

    /// Create a new assignment in this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:id/assignments`
    pub async fn create_assignment(&self, params: CreateAssignmentParams) -> Result<Assignment> {
        let form = wrap_params("assignment", &params);
        let mut a: Assignment = self
            .req()
            .post(&format!("courses/{}/assignments", self.id), &form)
            .await?;
        a.requester = Some(Arc::clone(self.req()));
        a.course_id = Some(self.id);
        Ok(a)
    }

    /// Stream all assignment groups in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/assignment_groups`
    pub fn get_assignment_groups(&self) -> PageStream<AssignmentGroup> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{}/assignment_groups", self.id),
            vec![],
            move |mut g: AssignmentGroup, req| {
                g.requester = Some(Arc::clone(&req));
                g.course_id = Some(course_id);
                g
            },
        )
    }

    /// Create a new assignment group in this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:id/assignment_groups`
    pub async fn create_assignment_group(
        &self,
        params: crate::resources::assignment::AssignmentGroupParams,
    ) -> Result<AssignmentGroup> {
        let form = wrap_params("assignment_group", &params);
        let mut g: AssignmentGroup = self
            .req()
            .post(&format!("courses/{}/assignment_groups", self.id), &form)
            .await?;
        g.requester = Some(Arc::clone(self.req()));
        g.course_id = Some(self.id);
        Ok(g)
    }

    /// Stream all sections in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/sections`
    pub fn get_sections(&self) -> PageStream<Section> {
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{}/sections", self.id),
            vec![],
            |mut s: Section, req| {
                s.requester = Some(Arc::clone(&req));
                s
            },
        )
    }

    /// Fetch a single section by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/sections/:section_id`
    pub async fn get_section(&self, section_id: u64) -> Result<Section> {
        let mut s: Section = self
            .req()
            .get(&format!("courses/{}/sections/{section_id}", self.id), &[])
            .await?;
        s.requester = self.requester.clone();
        Ok(s)
    }

    /// Stream all enrollments in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/enrollments`
    pub fn get_enrollments(&self) -> PageStream<Enrollment> {
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{}/enrollments", self.id),
            vec![],
            |mut e: Enrollment, req| {
                e.requester = Some(Arc::clone(&req));
                e
            },
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
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{}/quizzes", self.id),
            vec![],
            move |mut q: Quiz, req| {
                q.requester = Some(Arc::clone(&req));
                q.course_id = Some(course_id);
                q
            },
        )
    }

    /// Fetch a single quiz.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/quizzes/:quiz_id`
    pub async fn get_quiz(&self, quiz_id: u64) -> Result<Quiz> {
        let mut q: Quiz = self
            .req()
            .get(&format!("courses/{}/quizzes/{quiz_id}", self.id), &[])
            .await?;
        q.requester = Some(Arc::clone(self.req()));
        q.course_id = Some(self.id);
        Ok(q)
    }

    /// Create a new quiz in this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:id/quizzes`
    pub async fn create_quiz(&self, params: CreateQuizParams) -> Result<Quiz> {
        let form = wrap_params("quiz", &params);
        let mut q: Quiz = self
            .req()
            .post(&format!("courses/{}/quizzes", self.id), &form)
            .await?;
        q.requester = Some(Arc::clone(self.req()));
        q.course_id = Some(self.id);
        Ok(q)
    }

    /// Stream all modules in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/modules`
    pub fn get_modules(&self) -> PageStream<Module> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{}/modules", self.id),
            vec![],
            move |mut m: Module, req| {
                m.requester = Some(Arc::clone(&req));
                m.course_id = Some(course_id);
                m
            },
        )
    }

    /// Fetch a single module.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/modules/:module_id`
    pub async fn get_module(&self, module_id: u64) -> Result<Module> {
        let mut m: Module = self
            .req()
            .get(&format!("courses/{}/modules/{module_id}", self.id), &[])
            .await?;
        m.requester = Some(Arc::clone(self.req()));
        m.course_id = Some(self.id);
        Ok(m)
    }

    /// Create a new module in this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:id/modules`
    pub async fn create_module(
        &self,
        params: crate::resources::module::CreateModuleParams,
    ) -> Result<Module> {
        let form = wrap_params("module", &params);
        let mut m: Module = self
            .req()
            .post(&format!("courses/{}/modules", self.id), &form)
            .await?;
        m.requester = Some(Arc::clone(self.req()));
        m.course_id = Some(self.id);
        Ok(m)
    }

    /// Stream all pages in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/pages`
    pub fn get_pages(&self) -> PageStream<Page> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{}/pages", self.id),
            vec![],
            move |mut p: Page, req| {
                p.requester = Some(Arc::clone(&req));
                p.course_id = Some(course_id);
                p
            },
        )
    }

    /// Fetch a single page by URL slug or ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/pages/:url_or_id`
    pub async fn get_page(&self, url_or_id: &str) -> Result<Page> {
        let mut page: Page = self
            .req()
            .get(&format!("courses/{}/pages/{url_or_id}", self.id), &[])
            .await?;
        page.requester = self.requester.clone();
        page.course_id = Some(self.id);
        Ok(page)
    }

    /// Stream all discussion topics in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/discussion_topics`
    pub fn get_discussion_topics(&self) -> PageStream<DiscussionTopic> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{}/discussion_topics", self.id),
            vec![],
            move |mut t: DiscussionTopic, req| {
                t.requester = Some(Arc::clone(&req));
                t.course_id_ctx = Some(course_id);
                t
            },
        )
    }

    /// Fetch a single discussion topic.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/discussion_topics/:topic_id`
    pub async fn get_discussion_topic(&self, topic_id: u64) -> Result<DiscussionTopic> {
        let mut t: DiscussionTopic = self
            .req()
            .get(
                &format!("courses/{}/discussion_topics/{topic_id}", self.id),
                &[],
            )
            .await?;
        t.requester = Some(Arc::clone(self.req()));
        t.course_id_ctx = Some(self.id);
        Ok(t)
    }

    /// Create a new discussion topic in this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:id/discussion_topics`
    pub async fn create_discussion_topic(
        &self,
        params: crate::resources::discussion_topic::UpdateDiscussionParams,
    ) -> Result<DiscussionTopic> {
        let form = wrap_params("discussion_topic", &params);
        let mut t: DiscussionTopic = self
            .req()
            .post(&format!("courses/{}/discussion_topics", self.id), &form)
            .await?;
        t.requester = Some(Arc::clone(self.req()));
        t.course_id_ctx = Some(self.id);
        Ok(t)
    }

    /// Stream all files in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/files`
    pub fn get_files(&self) -> PageStream<File> {
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{}/files", self.id),
            vec![],
            |mut f: File, req| {
                f.requester = Some(Arc::clone(&req));
                f
            },
        )
    }

    /// Stream all tabs in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/tabs`
    pub fn get_tabs(&self) -> PageStream<Tab> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{}/tabs", self.id),
            vec![],
            move |mut t: Tab, req| {
                t.requester = Some(Arc::clone(&req));
                t.course_id = Some(course_id);
                t
            },
        )
    }

    /// Stream all collaborations in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/collaborations`
    pub fn get_collaborations(&self) -> PageStream<Collaboration> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/collaborations"),
            vec![],
            {
                let req = Arc::clone(self.req());
                move |mut c: Collaboration, _| {
                    c.requester = Some(Arc::clone(&req));
                    c
                }
            },
        )
    }

    /// Stream all groups in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/groups`
    pub fn get_groups(&self) -> PageStream<Group> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/groups"),
            vec![],
            |mut g: Group, req| {
                g.requester = Some(Arc::clone(&req));
                g
            },
        )
    }

    /// Stream all group categories in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/group_categories`
    pub fn get_group_categories(&self) -> PageStream<GroupCategory> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/group_categories"),
            vec![],
            |mut gc: GroupCategory, req| {
                gc.requester = Some(Arc::clone(&req));
                gc
            },
        )
    }

    /// Create a group category in this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:id/group_categories`
    pub async fn create_group_category(
        &self,
        params: crate::resources::group::GroupCategoryParams,
    ) -> Result<GroupCategory> {
        let form = wrap_params("group_category", &params);
        let mut gc: GroupCategory = self
            .req()
            .post(&format!("courses/{}/group_categories", self.id), &form)
            .await?;
        gc.requester = Some(Arc::clone(self.req()));
        Ok(gc)
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

    /// Fetch the root outcome group for this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/root_outcome_group`
    pub async fn get_root_outcome_group(&self) -> Result<OutcomeGroup> {
        let mut group: OutcomeGroup = self
            .req()
            .get(&format!("courses/{}/root_outcome_group", self.id), &[])
            .await?;
        group.requester = self.requester.clone();
        Ok(group)
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
    // Custom Gradebook Columns
    // -------------------------------------------------------------------------

    /// Stream all custom gradebook columns for this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/custom_gradebook_columns`
    pub fn get_custom_columns(&self) -> PageStream<CustomGradebookColumn> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/custom_gradebook_columns"),
            vec![],
            move |mut col: CustomGradebookColumn, req| {
                col.requester = Some(Arc::clone(&req));
                col.course_id = Some(course_id);
                col
            },
        )
    }

    /// Create a new custom gradebook column in this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:id/custom_gradebook_columns`
    pub async fn create_custom_column(
        &self,
        params: CustomGradebookColumnParams,
    ) -> Result<CustomGradebookColumn> {
        let form = wrap_params("column", &params);
        let mut col: CustomGradebookColumn = self
            .req()
            .post(
                &format!("courses/{}/custom_gradebook_columns", self.id),
                &form,
            )
            .await?;
        col.requester = self.requester.clone();
        col.course_id = Some(self.id);
        Ok(col)
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

    // -------------------------------------------------------------------------
    // Grading Periods
    // -------------------------------------------------------------------------

    /// Stream all grading periods for this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/grading_periods`
    pub fn get_grading_periods(&self) -> PageStream<GradingPeriod> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{course_id}/grading_periods"),
            vec![],
            move |mut gp: GradingPeriod, req| {
                gp.requester = Some(Arc::clone(&req));
                gp.course_id = Some(course_id);
                gp
            },
        )
    }

    // -------------------------------------------------------------------------
    // Grading Standards
    // -------------------------------------------------------------------------

    /// Stream all grading standards for this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/grading_standards`
    pub fn get_grading_standards(&self) -> PageStream<GradingStandard> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/grading_standards", self.id),
            vec![],
        )
    }

    /// Create a grading standard for this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/grading_standards`
    pub async fn create_grading_standard(
        &self,
        params: crate::resources::grading_standard::GradingStandardParams,
    ) -> Result<GradingStandard> {
        let form = wrap_params("grading_scheme_entry", &params.grading_scheme_entry)
            .into_iter()
            .chain([("title".into(), params.title)])
            .collect::<Vec<_>>();
        self.req()
            .post(&format!("courses/{}/grading_standards", self.id), &form)
            .await
    }

    // -------------------------------------------------------------------------
    // Content Exports
    // -------------------------------------------------------------------------

    /// Fetch a single content export by ID.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/content_exports/:id`
    pub async fn get_content_export(&self, export_id: u64) -> Result<ContentExport> {
        self.req()
            .get(
                &format!("courses/{}/content_exports/{export_id}", self.id),
                &[],
            )
            .await
    }

    /// Stream all content exports for this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/content_exports`
    pub fn get_content_exports(&self) -> PageStream<ContentExport> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/content_exports", self.id),
            vec![],
        )
    }

    /// Create a content export for this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/content_exports`
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
            .post(&format!("courses/{}/content_exports", self.id), &form)
            .await
    }

    // -------------------------------------------------------------------------
    // Grade Change Log
    // -------------------------------------------------------------------------

    /// Stream grade change audit events for this course.
    ///
    /// The Canvas API wraps the array in `{ "events": [...] }`; `PageStream`
    /// handles this automatically.
    ///
    /// # Canvas API
    /// `GET /api/v1/audit/grade_change/courses/:course_id`
    pub fn get_grade_change_events(&self) -> PageStream<GradeChangeEvent> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("audit/grade_change/courses/{}", self.id),
            vec![],
        )
    }

    // -------------------------------------------------------------------------
    // Features
    // -------------------------------------------------------------------------

    /// Stream all feature flags for this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/features`
    pub fn get_features(&self) -> PageStream<Feature> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/features", self.id),
            vec![],
        )
    }

    /// Fetch a specific feature flag for this course by feature name.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/features/flags/:feature`
    pub async fn get_feature_flag(&self, feature: &str) -> Result<FeatureFlag> {
        let mut ff: FeatureFlag = self
            .req()
            .get(
                &format!("courses/{}/features/flags/{feature}", self.id),
                &[],
            )
            .await?;
        ff.requester = self.requester.clone();
        Ok(ff)
    }

    /// List all enabled feature names for this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/features/enabled`
    pub async fn get_enabled_features(&self) -> Result<Vec<String>> {
        self.req()
            .get(&format!("courses/{}/features/enabled", self.id), &[])
            .await
    }

    // -------------------------------------------------------------------------
    // LTI Resource Links
    // -------------------------------------------------------------------------

    /// Stream all LTI resource links in this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/lti_resource_links`
    pub fn get_lti_resource_links(&self) -> PageStream<LtiResourceLink> {
        PageStream::new(
            Arc::clone(self.req()),
            &format!("courses/{}/lti_resource_links", self.id),
            vec![],
        )
    }

    /// Fetch a single LTI resource link.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:course_id/lti_resource_links/:id`
    pub async fn get_lti_resource_link(&self, link_id: u64) -> Result<LtiResourceLink> {
        self.req()
            .get(
                &format!("courses/{}/lti_resource_links/{link_id}", self.id),
                &[],
            )
            .await
    }

    /// Create a new LTI resource link in this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:course_id/lti_resource_links`
    pub async fn create_lti_resource_link(
        &self,
        params: CreateLtiResourceLinkParams,
    ) -> Result<LtiResourceLink> {
        let form = crate::params::flatten_params(&serde_json::to_value(&params).unwrap());
        self.req()
            .post(&format!("courses/{}/lti_resource_links", self.id), &form)
            .await
    }

    /// Conclude (soft-delete) this course.
    ///
    /// # Canvas API
    /// `DELETE /api/v1/courses/:id?event=conclude`
    pub async fn conclude(&self) -> Result<serde_json::Value> {
        self.req()
            .delete(
                &format!("courses/{}", self.id),
                &[("event".to_string(), "conclude".to_string())],
            )
            .await
    }

    /// Reset this course to a blank state (removes all content).
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:id/reset_content`
    pub async fn reset(&self) -> Result<Course> {
        let mut c: Course = self
            .req()
            .post(&format!("courses/{}/reset_content", self.id), &[])
            .await?;
        c.requester = Some(Arc::clone(self.req()));
        Ok(c)
    }

    /// Get this course's settings.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/settings`
    pub async fn get_settings(&self) -> Result<serde_json::Value> {
        self.req()
            .get(&format!("courses/{}/settings", self.id), &[])
            .await
    }

    /// Update this course's settings.
    ///
    /// # Canvas API
    /// `PUT /api/v1/courses/:id/settings`
    pub async fn update_settings(&self, params: &[(String, String)]) -> Result<serde_json::Value> {
        self.req()
            .put(&format!("courses/{}/settings", self.id), params)
            .await
    }

    /// Get the late policy for this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/late_policy`
    pub async fn get_late_policy(&self) -> Result<serde_json::Value> {
        self.req()
            .get(&format!("courses/{}/late_policy", self.id), &[])
            .await
    }

    /// Stream multiple submissions for this course.
    ///
    /// # Canvas API
    /// `GET /api/v1/courses/:id/students/submissions`
    pub fn get_multiple_submissions(&self) -> PageStream<crate::resources::submission::Submission> {
        let course_id = self.id;
        PageStream::new_with_injector(
            Arc::clone(self.req()),
            &format!("courses/{}/students/submissions", self.id),
            vec![],
            move |mut s: crate::resources::submission::Submission, req| {
                s.requester = Some(Arc::clone(&req));
                s.course_id = Some(course_id);
                s
            },
        )
    }

    /// Enroll a user in this course.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:id/enrollments`
    pub async fn enroll_user(
        &self,
        user_id: u64,
        enrollment_type: &str,
    ) -> Result<crate::resources::enrollment::Enrollment> {
        let params = vec![
            ("enrollment[user_id]".to_string(), user_id.to_string()),
            ("enrollment[type]".to_string(), enrollment_type.to_string()),
        ];
        let mut e: crate::resources::enrollment::Enrollment = self
            .req()
            .post(&format!("courses/{}/enrollments", self.id), &params)
            .await?;
        e.requester = Some(Arc::clone(self.req()));
        Ok(e)
    }

    /// Bulk-update grades for this course asynchronously.
    ///
    /// # Canvas API
    /// `POST /api/v1/courses/:id/submissions/update_grades`
    pub async fn submissions_bulk_update(
        &self,
        params: &[(String, String)],
    ) -> Result<crate::resources::progress::Progress> {
        let mut p: crate::resources::progress::Progress = self
            .req()
            .post(
                &format!("courses/{}/submissions/update_grades", self.id),
                params,
            )
            .await?;
        p.requester = Some(Arc::clone(self.req()));
        Ok(p)
    }
}
