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
