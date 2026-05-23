use canvas_lms_api::resources::external_tool::ExternalToolParams;
use canvas_lms_api::resources::params::assignment_params::CreateAssignmentParams;
use canvas_lms_api::resources::params::course_params::UpdateCourseParams;
use canvas_lms_api::resources::params::quiz_params::CreateQuizParams;
use canvas_lms_api::resources::rubric::RubricParams;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn make_course(server: &MockServer) -> canvas_lms_api::resources::course::Course {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "Test Course",
            "course_code": "TEST-101"
        })))
        .mount(server)
        .await;
    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    canvas.get_course(1).await.unwrap()
}

#[tokio::test]
async fn test_create_assignment() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/assignments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "course_id": 1,
            "name": "Homework 1"
        })))
        .mount(&server)
        .await;

    let params = CreateAssignmentParams {
        name: "Homework 1".to_string(),
        ..Default::default()
    };
    let assignment = course.create_assignment(params).await.unwrap();

    assert_eq!(assignment.id, 10);
    assert_eq!(assignment.name.as_deref(), Some("Homework 1"));
}

#[tokio::test]
async fn test_get_assignment() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignments/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "course_id": 1,
            "name": "Essay"
        })))
        .mount(&server)
        .await;

    let assignment = course.get_assignment(5).await.unwrap();

    assert_eq!(assignment.id, 5);
    assert_eq!(assignment.name.as_deref(), Some("Essay"));
}

#[tokio::test]
async fn test_get_assignments() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "course_id": 1, "name": "Assignment A"},
            {"id": 2, "course_id": 1, "name": "Assignment B"}
        ])))
        .mount(&server)
        .await;

    let assignments = course.get_assignments().collect_all().await.unwrap();

    assert_eq!(assignments.len(), 2);
    assert_eq!(assignments[0].id, 1);
    assert_eq!(assignments[1].id, 2);
}

#[tokio::test]
async fn test_update_course() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "Updated Course",
            "course_code": "TEST-101"
        })))
        .mount(&server)
        .await;

    let params = UpdateCourseParams {
        name: Some("Updated Course".to_string()),
        ..Default::default()
    };
    let updated = course.update(params).await.unwrap();

    assert_eq!(updated.id, 1);
    assert_eq!(updated.name.as_deref(), Some("Updated Course"));
}

#[tokio::test]
async fn test_delete_course_method() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "Test Course",
            "workflow_state": "deleted"
        })))
        .mount(&server)
        .await;

    let deleted = course.delete().await.unwrap();

    assert_eq!(deleted.id, 1);
    assert!(matches!(
        deleted.workflow_state,
        Some(canvas_lms_api::resources::types::WorkflowState::Deleted)
    ));
}

#[tokio::test]
async fn test_get_section() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/sections/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
            "course_id": 1,
            "name": "Section A"
        })))
        .mount(&server)
        .await;

    let section = course.get_section(3).await.unwrap();

    assert_eq!(section.id, 3);
    assert_eq!(section.name.as_deref(), Some("Section A"));
}

#[tokio::test]
async fn test_get_sections() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/sections"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "course_id": 1, "name": "Section A"},
            {"id": 2, "course_id": 1, "name": "Section B"}
        ])))
        .mount(&server)
        .await;

    let sections = course.get_sections().collect_all().await.unwrap();

    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].id, 1);
}

#[tokio::test]
async fn test_get_enrollments() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/enrollments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 101, "course_id": 1, "user_id": 42},
            {"id": 102, "course_id": 1, "user_id": 43}
        ])))
        .mount(&server)
        .await;

    let enrollments = course.get_enrollments().collect_all().await.unwrap();

    assert_eq!(enrollments.len(), 2);
    assert_eq!(enrollments[0].id, 101);
}

#[tokio::test]
async fn test_get_users() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 42, "name": "Alice"},
            {"id": 43, "name": "Bob"}
        ])))
        .mount(&server)
        .await;

    let users = course.get_users().collect_all().await.unwrap();

    assert_eq!(users.len(), 2);
    assert_eq!(users[0].id, 42);
}

#[tokio::test]
async fn test_get_quiz() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/quizzes/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "course_id": 1,
            "title": "Midterm Quiz"
        })))
        .mount(&server)
        .await;

    let quiz = course.get_quiz(7).await.unwrap();

    assert_eq!(quiz.id, 7);
    assert_eq!(quiz.title.as_deref(), Some("Midterm Quiz"));
}

#[tokio::test]
async fn test_get_quizzes() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/quizzes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "course_id": 1, "title": "Quiz 1"},
            {"id": 2, "course_id": 1, "title": "Quiz 2"}
        ])))
        .mount(&server)
        .await;

    let quizzes = course.get_quizzes().collect_all().await.unwrap();

    assert_eq!(quizzes.len(), 2);
    assert_eq!(quizzes[0].id, 1);
}

#[tokio::test]
async fn test_create_quiz() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/quizzes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20,
            "course_id": 1,
            "title": "Final Exam"
        })))
        .mount(&server)
        .await;

    let params = CreateQuizParams {
        title: "Final Exam".to_string(),
        ..Default::default()
    };
    let quiz = course.create_quiz(params).await.unwrap();

    assert_eq!(quiz.id, 20);
    assert_eq!(quiz.title.as_deref(), Some("Final Exam"));
}

#[tokio::test]
async fn test_get_module() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/modules/2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 2,
            "course_id": 1,
            "name": "Week 1"
        })))
        .mount(&server)
        .await;

    let module = course.get_module(2).await.unwrap();

    assert_eq!(module.id, 2);
    assert_eq!(module.name.as_deref(), Some("Week 1"));
}

#[tokio::test]
async fn test_get_modules() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/modules"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "course_id": 1, "name": "Week 1"},
            {"id": 2, "course_id": 1, "name": "Week 2"}
        ])))
        .mount(&server)
        .await;

    let modules = course.get_modules().collect_all().await.unwrap();

    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].id, 1);
}

#[tokio::test]
async fn test_get_page() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/pages/syllabus"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "page_id": 5,
            "url": "syllabus",
            "title": "Syllabus"
        })))
        .mount(&server)
        .await;

    let page = course.get_page("syllabus").await.unwrap();

    assert_eq!(page.page_id, Some(5));
    assert_eq!(page.title.as_deref(), Some("Syllabus"));
}

#[tokio::test]
async fn test_get_pages() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/pages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"page_id": 1, "url": "intro", "title": "Introduction"},
            {"page_id": 2, "url": "syllabus", "title": "Syllabus"}
        ])))
        .mount(&server)
        .await;

    let pages = course.get_pages().collect_all().await.unwrap();

    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].page_id, Some(1));
}

#[tokio::test]
async fn test_get_discussion_topic() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/discussion_topics/11"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 11,
            "course_id": 1,
            "title": "Week 1 Discussion"
        })))
        .mount(&server)
        .await;

    let topic = course.get_discussion_topic(11).await.unwrap();

    assert_eq!(topic.id, 11);
    assert_eq!(topic.title.as_deref(), Some("Week 1 Discussion"));
}

#[tokio::test]
async fn test_get_discussion_topics() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/discussion_topics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 11, "course_id": 1, "title": "Discussion A"},
            {"id": 12, "course_id": 1, "title": "Discussion B"}
        ])))
        .mount(&server)
        .await;

    let topics = course.get_discussion_topics().collect_all().await.unwrap();

    assert_eq!(topics.len(), 2);
    assert_eq!(topics[0].id, 11);
}

#[tokio::test]
async fn test_get_files() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 201, "display_name": "lecture1.pdf"},
            {"id": 202, "display_name": "lecture2.pdf"}
        ])))
        .mount(&server)
        .await;

    let files = course.get_files().collect_all().await.unwrap();

    assert_eq!(files.len(), 2);
    assert_eq!(files[0].id, 201);
}

#[tokio::test]
async fn test_get_tabs() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/tabs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "home", "label": "Home", "position": 1},
            {"id": "assignments", "label": "Assignments", "position": 2}
        ])))
        .mount(&server)
        .await;

    let tabs = course.get_tabs().collect_all().await.unwrap();

    assert_eq!(tabs.len(), 2);
    assert_eq!(tabs[0].id.as_deref(), Some("home"));
}

#[tokio::test]
async fn test_get_groups() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 301, "name": "Group A", "course_id": 1},
            {"id": 302, "name": "Group B", "course_id": 1}
        ])))
        .mount(&server)
        .await;

    let groups = course.get_groups().collect_all().await.unwrap();

    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].id, 301);
}
// ---- External Tools ----

#[tokio::test]
async fn test_get_external_tool_on_course() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/external_tools/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "name": "Canvas Studio",
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let tool = course.get_external_tool(5).await.unwrap();
    assert_eq!(tool.id, 5);
    assert_eq!(tool.name.as_deref(), Some("Canvas Studio"));
}

#[tokio::test]
async fn test_get_external_tools_on_course() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/external_tools"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 5, "name": "Tool A", "course_id": 1},
            {"id": 6, "name": "Tool B", "course_id": 1}
        ])))
        .mount(&server)
        .await;

    let tools = course.get_external_tools().collect_all().await.unwrap();
    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].id, 5);
}

#[tokio::test]
async fn test_create_external_tool_on_course() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/external_tools"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "name": "New Tool",
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let params = ExternalToolParams {
        name: Some("New Tool".to_string()),
        ..Default::default()
    };
    let tool = course.create_external_tool(params).await.unwrap();
    assert_eq!(tool.id, 7);
    assert_eq!(tool.name.as_deref(), Some("New Tool"));
}

// ---- Rubrics ----

#[tokio::test]
async fn test_get_rubric_on_course() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/rubrics/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3,
            "title": "Essay Rubric",
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let rubric = course.get_rubric(3).await.unwrap();
    assert_eq!(rubric.id, 3);
    assert_eq!(rubric.title.as_deref(), Some("Essay Rubric"));
}

#[tokio::test]
async fn test_get_rubrics_on_course() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/rubrics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 3, "title": "Rubric A", "course_id": 1},
            {"id": 4, "title": "Rubric B", "course_id": 1}
        ])))
        .mount(&server)
        .await;

    let rubrics = course.get_rubrics().collect_all().await.unwrap();
    assert_eq!(rubrics.len(), 2);
    assert_eq!(rubrics[0].id, 3);
}

#[tokio::test]
async fn test_create_rubric_on_course() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/rubrics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "title": "New Rubric",
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let params = RubricParams {
        title: Some("New Rubric".to_string()),
        ..Default::default()
    };
    let rubric = course.create_rubric(params).await.unwrap();
    assert_eq!(rubric.id, 10);
    assert_eq!(rubric.title.as_deref(), Some("New Rubric"));
}

// ---- Blueprint ----

#[tokio::test]
async fn test_get_blueprint() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/blueprint_templates/default"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "course_id": 1,
            "associated_course_count": 5
        })))
        .mount(&server)
        .await;

    let tmpl = course.get_blueprint("default").await.unwrap();
    assert_eq!(tmpl.id, 1);
    assert_eq!(tmpl.associated_course_count, Some(5));
}

#[tokio::test]
async fn test_get_blueprint_subscriptions() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/blueprint_subscriptions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "template_id": 1, "course_id": 1}
        ])))
        .mount(&server)
        .await;

    let subs = course
        .get_blueprint_subscriptions()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].id, 10);
}

// ---- Content Migrations ----

#[tokio::test]
async fn test_get_content_migration_on_course() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/content_migrations/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99,
            "course_id": 1,
            "migration_type": "common_cartridge_importer",
            "workflow_state": "completed"
        })))
        .mount(&server)
        .await;

    let migration = course.get_content_migration(99).await.unwrap();
    assert_eq!(migration.id, 99);
    assert_eq!(migration.workflow_state.as_deref(), Some("completed"));
}

#[tokio::test]
async fn test_get_content_migrations_on_course() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/content_migrations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "course_id": 1, "migration_type": "common_cartridge_importer", "workflow_state": "completed"}
        ])))
        .mount(&server)
        .await;

    let migrations = course.get_content_migrations().collect_all().await.unwrap();
    assert_eq!(migrations.len(), 1);
    assert_eq!(migrations[0].id, 1);
}

#[tokio::test]
async fn test_create_content_migration_on_course() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/content_migrations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 100,
            "course_id": 1,
            "migration_type": "common_cartridge_importer",
            "workflow_state": "pre_processing"
        })))
        .mount(&server)
        .await;

    let migration = course
        .create_content_migration("common_cartridge_importer", &[])
        .await
        .unwrap();
    assert_eq!(migration.id, 100);
    assert_eq!(migration.workflow_state.as_deref(), Some("pre_processing"));
}

// ---- Outcome Groups ----

#[tokio::test]
async fn test_get_outcome_group_links_on_course() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/outcome_group_links"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"context_id": 1, "context_type": "Course"},
            {"context_id": 1, "context_type": "Course"}
        ])))
        .mount(&server)
        .await;

    let links = course
        .get_outcome_group_links()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(links.len(), 2);
}

#[tokio::test]
async fn test_get_outcome_group_on_course() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/outcome_groups/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20,
            "title": "Writing Skills",
            "context_id": 1,
            "context_type": "Course"
        })))
        .mount(&server)
        .await;

    let group = course.get_outcome_group(20).await.unwrap();
    assert_eq!(group.id, 20);
    assert_eq!(group.title.as_deref(), Some("Writing Skills"));
}

// ---- Gradebook History ----

#[tokio::test]
async fn test_get_gradebook_history_dates() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/gradebook_history/days"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"date": "2026-05-01", "graders": 3},
            {"date": "2026-05-02", "graders": 2}
        ])))
        .mount(&server)
        .await;

    let days = course
        .get_gradebook_history_dates()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(days.len(), 2);
    assert_eq!(days[0].date.as_deref(), Some("2026-05-01"));
    assert_eq!(days[0].graders, Some(3));
}

#[tokio::test]
async fn test_get_gradebook_history_details() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/gradebook_history/2026-05-01"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 42, "name": "Prof. Smith"},
            {"id": 43, "name": "Prof. Jones"}
        ])))
        .mount(&server)
        .await;

    let graders = course
        .get_gradebook_history_details("2026-05-01")
        .collect_all()
        .await
        .unwrap();
    assert_eq!(graders.len(), 2);
    assert_eq!(graders[0].id, Some(42));
    assert_eq!(graders[0].name.as_deref(), Some("Prof. Smith"));
}

#[tokio::test]
async fn test_get_submission_history() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v1/courses/1/gradebook_history/2026-05-01/graders/42/assignments/10/submissions",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"submission_id": 101, "versions": []},
            {"submission_id": 102, "versions": []}
        ])))
        .mount(&server)
        .await;

    let history = course
        .get_submission_history("2026-05-01", 42, 10)
        .collect_all()
        .await
        .unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].submission_id, Some(101));
}

#[tokio::test]
async fn test_get_uncollated_submissions() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/gradebook_history/feed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "assignment_id": 10, "user_id": 99, "grade": "A"},
            {"id": 2, "assignment_id": 11, "user_id": 99, "grade": "B+"}
        ])))
        .mount(&server)
        .await;

    let versions = course
        .get_uncollated_submissions()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].assignment_id, Some(10));
    assert_eq!(versions[0].grade.as_deref(), Some("A"));
}

#[tokio::test]
async fn test_get_rubric_association() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/rubric_associations/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "rubric_id": 3,
            "association_id": 1,
            "association_type": "Course",
            "course_id": 1,
            "purpose": "grading"
        })))
        .mount(&server)
        .await;

    let assoc = course.get_rubric_association(7).await.unwrap();
    assert_eq!(assoc.id, 7);
    assert_eq!(assoc.rubric_id, Some(3));
    assert_eq!(assoc.purpose.as_deref(), Some("grading"));
}

#[tokio::test]
async fn test_get_rubric_associations() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/rubric_associations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 7, "rubric_id": 3, "association_id": 1, "association_type": "Course", "course_id": 1},
            {"id": 8, "rubric_id": 4, "association_id": 10, "association_type": "Assignment", "course_id": 1}
        ])))
        .mount(&server)
        .await;

    let assocs = course
        .get_rubric_associations()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(assocs.len(), 2);
    assert_eq!(assocs[0].id, 7);
    assert_eq!(assocs[1].association_type.as_deref(), Some("Assignment"));
}

#[tokio::test]
async fn test_create_rubric_association() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/rubric_associations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 9,
            "rubric_id": 3,
            "association_id": 10,
            "association_type": "Assignment",
            "course_id": 1,
            "purpose": "grading"
        })))
        .mount(&server)
        .await;

    let assoc = course
        .create_rubric_association(&[
            ("rubric_association[rubric_id]".to_string(), "3".to_string()),
            (
                "rubric_association[association_id]".to_string(),
                "10".to_string(),
            ),
            (
                "rubric_association[association_type]".to_string(),
                "Assignment".to_string(),
            ),
            (
                "rubric_association[purpose]".to_string(),
                "grading".to_string(),
            ),
        ])
        .await
        .unwrap();
    assert_eq!(assoc.id, 9);
    assert_eq!(assoc.rubric_id, Some(3));
    assert_eq!(assoc.purpose.as_deref(), Some("grading"));
    assert_eq!(assoc.course_id, Some(1));
}

#[tokio::test]
async fn test_course_import_outcomes() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/outcome_imports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "course_id": 1,
            "workflow_state": "created"
        })))
        .mount(&server)
        .await;

    let import = course.import_outcomes(&[]).await.unwrap();
    assert_eq!(import.id, 42);
    assert_eq!(import.workflow_state.as_deref(), Some("created"));
}

#[tokio::test]
async fn test_course_get_outcome_import_status() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/outcome_imports/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "course_id": 1,
            "workflow_state": "succeeded"
        })))
        .mount(&server)
        .await;

    let import = course.get_outcome_import_status(42).await.unwrap();
    assert_eq!(import.id, 42);
    assert_eq!(import.workflow_state.as_deref(), Some("succeeded"));
}


#[tokio::test]
async fn test_get_single_grading_standard() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/grading_standards/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "title": "Letter Grade",
            "context_type": "Course"
        })))
        .mount(&server)
        .await;

    let gs = course.get_single_grading_standard(5).await.unwrap();
    assert_eq!(gs.title.as_deref(), Some("Letter Grade"));
}

#[tokio::test]
async fn test_get_assignment_overrides() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignments/overrides"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "assignment_id": 1, "title": "Section Override"},
            {"id": 11, "assignment_id": 2, "title": "Student Override"}
        ])))
        .mount(&server)
        .await;

    let overrides = course.get_assignment_overrides(&[1, 2]).collect_all().await.unwrap();
    assert_eq!(overrides.len(), 2);
    assert_eq!(overrides[0].id, 10);
}

#[tokio::test]
async fn test_create_assignment_overrides() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/assignments/overrides"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 20, "assignment_id": 1, "title": "Section Override"}
        ])))
        .mount(&server)
        .await;

    let params = vec![
        ("assignment_overrides[][assignment_id]".to_string(), "1".to_string()),
    ];
    let overrides = course.create_assignment_overrides(&params).await.unwrap();
    assert_eq!(overrides.len(), 1);
    assert_eq!(overrides[0].id, 20);
}

#[tokio::test]
async fn test_update_assignment_overrides() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/assignments/overrides"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 10, "assignment_id": 1, "title": "Updated Override"}
        ])))
        .mount(&server)
        .await;

    let params = vec![
        ("assignment_overrides[][id]".to_string(), "10".to_string()),
    ];
    let overrides = course.update_assignment_overrides(&params).await.unwrap();
    assert_eq!(overrides.len(), 1);
    assert_eq!(overrides[0].id, 10);
}

#[tokio::test]
async fn test_get_assignments_for_group() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignment_groups/5/assignments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 100, "name": "Essay 1", "assignment_group_id": 5},
            {"id": 101, "name": "Essay 2", "assignment_group_id": 5}
        ])))
        .mount(&server)
        .await;

    let assignments = course.get_assignments_for_group(5).collect_all().await.unwrap();
    assert_eq!(assignments.len(), 2);
    assert_eq!(assignments[0].id, 100);
    assert_eq!(assignments[0].name.as_deref(), Some("Essay 1"));
}

#[tokio::test]
async fn test_get_all_outcome_links_in_context() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/outcome_group_links"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"outcome": {"id": 1, "title": "Critical Thinking"}, "context_type": "Course"}
        ])))
        .mount(&server)
        .await;

    let links = course.get_all_outcome_links_in_context().collect_all().await.unwrap();
    assert_eq!(links.len(), 1);
}

#[tokio::test]
async fn test_get_todo_items() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/todo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"type": "grading", "assignment": {"id": 1, "name": "Paper"}},
            {"type": "submitting", "assignment": {"id": 2, "name": "Quiz"}}
        ])))
        .mount(&server)
        .await;

    let items = course.get_todo_items().collect_all().await.unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["type"], "grading");
}

#[tokio::test]
async fn test_create_epub_export() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/epub_exports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "workflow_state": "created",
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let result = course.create_epub_export().await.unwrap();
    assert_eq!(result["id"], 7);
    assert_eq!(result["workflow_state"], "created");
}

#[tokio::test]
async fn test_get_epub_export() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/epub_exports/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "workflow_state": "generated",
            "course_id": 1
        })))
        .mount(&server)
        .await;

    let result = course.get_epub_export(7).await.unwrap();
    assert_eq!(result["id"], 7);
    assert_eq!(result["workflow_state"], "generated");
}

#[tokio::test]
async fn test_column_data_bulk_update() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/custom_gradebook_column_data"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "workflow_state": "queued",
            "completion": 0
        })))
        .mount(&server)
        .await;

    let params = vec![("column_data[][column_id]".to_string(), "1".to_string())];
    let progress = course.column_data_bulk_update(&params).await.unwrap();
    assert_eq!(progress.id, 10);
}

#[tokio::test]
async fn test_query_audit_by_course() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/audit/course/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"course_id": 1, "event_type": "created", "user_id": 5}
        ])))
        .mount(&server)
        .await;

    let events = course.query_audit_by_course().collect_all().await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event_type"], "created");
}

#[tokio::test]
async fn test_get_course_level_assignment_data() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/analytics/assignments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"assignment_id": 1, "title": "Essay", "due_at": "2024-03-01T00:00:00Z"}
        ])))
        .mount(&server)
        .await;

    let result = course.get_course_level_assignment_data().await.unwrap();
    assert!(result.is_array());
}

#[tokio::test]
async fn test_get_course_level_participation_data() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/analytics/activity"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"date": "2024-01-01", "participations": 10, "views": 50}
        ])))
        .mount(&server)
        .await;

    let result = course.get_course_level_participation_data().await.unwrap();
    assert!(result.is_array());
}

#[tokio::test]
async fn test_get_course_level_student_summary_data() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/analytics/student_summaries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"user_id": 42, "participations": 5, "page_views": 25}
        ])))
        .mount(&server)
        .await;

    let summaries = course.get_course_level_student_summary_data().collect_all().await.unwrap();
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0]["user_id"], 42);
}

#[tokio::test]
async fn test_get_user_in_a_course_level_assignment_data() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/analytics/users/42/assignments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"assignment_id": 1, "submission": {"score": 95, "submitted_at": "2024-02-01T00:00:00Z"}}
        ])))
        .mount(&server)
        .await;

    let result = course
        .get_user_in_a_course_level_assignment_data(42)
        .await
        .unwrap();
    assert!(result.is_array());
}

#[tokio::test]
async fn test_get_user_in_a_course_level_messaging_data() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/analytics/users/42/communication"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"date": "2024-01-15", "sent": 2, "received": 3}
        ])))
        .mount(&server)
        .await;

    let result = course
        .get_user_in_a_course_level_messaging_data(42)
        .await
        .unwrap();
    assert!(result.is_array());
}

#[tokio::test]
async fn test_get_user_in_a_course_level_participation_data() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/analytics/users/42/activity"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "participations": [{"created_at": "2024-01-01T10:00:00Z"}],
            "page_views": {"2024-01-01": {"count": 5}}
        })))
        .mount(&server)
        .await;

    let result = course
        .get_user_in_a_course_level_participation_data(42)
        .await
        .unwrap();
    assert!(result.is_object());
}

#[tokio::test]
async fn test_smartsearch() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/smartsearch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "title": "Week 1 Notes", "body": "Introduction to the course"}
        ])))
        .mount(&server)
        .await;

    let results = course.smartsearch("introduction").collect_all().await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["title"], "Week 1 Notes");
}

#[tokio::test]
async fn test_get_quiz_overrides() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/quizzes/assignment_overrides"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"quiz_id": 5, "overrides": [{"id": 1, "title": "Section Override"}]}
        ])))
        .mount(&server)
        .await;

    let overrides = course.get_quiz_overrides().collect_all().await.unwrap();
    assert_eq!(overrides.len(), 1);
    assert_eq!(overrides[0]["quiz_id"], 5);
}

#[tokio::test]
async fn test_set_quiz_extensions() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/quiz_extensions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "quiz_extensions": [
                {"user_id": 42, "extra_time": 30, "extra_attempts": 1}
            ]
        })))
        .mount(&server)
        .await;

    let params = vec![
        ("quiz_extensions[][user_id]".to_string(), "42".to_string()),
        ("quiz_extensions[][extra_time]".to_string(), "30".to_string()),
    ];
    let result = course.set_quiz_extensions(&params).await.unwrap();
    assert!(result["quiz_extensions"].is_array());
}


#[tokio::test]
async fn test_course_get_file() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/files/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42, "display_name": "notes.pdf", "filename": "notes.pdf",
            "content_type": "application/pdf", "size": 2048
        })))
        .mount(&server)
        .await;

    let file = course.get_file(42).await.unwrap();
    assert_eq!(file.id, 42);
    assert_eq!(file.display_name.as_deref(), Some("notes.pdf"));
}

#[tokio::test]
async fn test_course_get_file_quota() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/files/quota"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "quota": 524288000,
            "quota_used": 10485760
        })))
        .mount(&server)
        .await;

    let quota = course.get_file_quota().await.unwrap();
    assert_eq!(quota["quota"], 524288000);
}

#[tokio::test]
async fn test_course_get_folder() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/folders/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7, "name": "Lecture Notes", "full_name": "course files/Lecture Notes"
        })))
        .mount(&server)
        .await;

    let folder = course.get_folder(7).await.unwrap();
    assert_eq!(folder.id, 7);
    assert_eq!(folder.name.as_deref(), Some("Lecture Notes"));
}

#[tokio::test]
async fn test_course_get_folders() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/folders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "course files"},
            {"id": 2, "name": "Submissions"}
        ])))
        .mount(&server)
        .await;

    let folders: Vec<_> = course.get_folders().collect_all().await.unwrap();
    assert_eq!(folders.len(), 2);
    assert_eq!(folders[0].id, 1);
}

#[tokio::test]
async fn test_course_create_folder() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/folders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99, "name": "New Folder", "full_name": "course files/New Folder"
        })))
        .mount(&server)
        .await;

    let folder = course
        .create_folder(&[("name".to_string(), "New Folder".to_string())])
        .await
        .unwrap();
    assert_eq!(folder.id, 99);
    assert_eq!(folder.name.as_deref(), Some("New Folder"));
}

#[tokio::test]
async fn test_course_create_page() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/pages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "url": "syllabus",
            "title": "Syllabus",
            "body": "<p>Welcome</p>"
        })))
        .mount(&server)
        .await;

    let page = course
        .create_page(&[
            ("wiki_page[title]".to_string(), "Syllabus".to_string()),
            ("wiki_page[body]".to_string(), "<p>Welcome</p>".to_string()),
        ])
        .await
        .unwrap();
    assert_eq!(page.title.as_deref(), Some("Syllabus"));
    assert_eq!(page.course_id, Some(1));
}

#[tokio::test]
async fn test_course_get_grading_period() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/grading_periods/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "grading_periods": [{"id": 5, "title": "Q1", "weight": 25.0}]
        })))
        .mount(&server)
        .await;

    let gp = course.get_grading_period(5).await.unwrap();
    assert_eq!(gp.id, 5);
    assert_eq!(gp.title.as_deref(), Some("Q1"));
}

#[tokio::test]
async fn test_course_get_assignment_group() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/assignment_groups/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 3, "name": "Homework", "group_weight": 40.0
        })))
        .mount(&server)
        .await;

    let ag = course.get_assignment_group(3).await.unwrap();
    assert_eq!(ag.id, 3);
    assert_eq!(ag.name.as_deref(), Some("Homework"));
    assert_eq!(ag.course_id, Some(1));
}

#[tokio::test]
async fn test_course_create_late_policy() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/courses/1/late_policy"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "late_policy": {"late_submission_deduction_enabled": true}
        })))
        .mount(&server)
        .await;

    let result = course
        .create_late_policy(&[(
            "late_policy[late_submission_deduction_enabled]".to_string(),
            "true".to_string(),
        )])
        .await
        .unwrap();
    assert_eq!(result["late_policy"]["late_submission_deduction_enabled"], true);
}

#[tokio::test]
async fn test_course_edit_late_policy() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("PATCH"))
        .and(path("/api/v1/courses/1/late_policy"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    course
        .edit_late_policy(&[(
            "late_policy[late_submission_minimum_percent]".to_string(),
            "50".to_string(),
        )])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_course_get_outcome_groups_in_context() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/outcome_groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "title": "Math Standards"},
            {"id": 2, "title": "Reading Standards"}
        ])))
        .mount(&server)
        .await;

    let groups: Vec<_> = course
        .get_outcome_groups_in_context()
        .collect_all()
        .await
        .unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].title.as_deref(), Some("Math Standards"));
}

#[tokio::test]
async fn test_course_get_outcome_result_rollups() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/outcome_rollups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "rollups": [{"links": {"user": "1"}, "scores": []}]
        })))
        .mount(&server)
        .await;

    let result = course.get_outcome_result_rollups().await.unwrap();
    assert!(result["rollups"].is_array());
}

#[tokio::test]
async fn test_course_get_outcome_results() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/outcome_results"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "score": 4.0, "links": {"user": "1"}},
            {"id": 2, "score": 3.0, "links": {"user": "2"}}
        ])))
        .mount(&server)
        .await;

    let results: Vec<_> = course.get_outcome_results().collect_all().await.unwrap();
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_course_remove_nickname() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/users/self/course_nicknames/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "course_id": 1,
            "name": "Test Course",
            "nickname": ""
        })))
        .mount(&server)
        .await;

    let result = course.remove_nickname().await.unwrap();
    assert_eq!(result["course_id"], 1);
}

#[tokio::test]
async fn test_course_resolve_path() {
    let server = MockServer::start().await;
    let course = make_course(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/folders/by_path/lectures/week1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "course files"},
            {"id": 2, "name": "lectures"},
            {"id": 3, "name": "week1"}
        ])))
        .mount(&server)
        .await;

    let folders: Vec<_> = course
        .resolve_path(Some("lectures/week1"))
        .collect_all()
        .await
        .unwrap();
    assert_eq!(folders.len(), 3);
    assert_eq!(folders[2].name.as_deref(), Some("week1"));
}
