use canvas_lms_api::resources::params::assignment_params::CreateAssignmentParams;
use canvas_lms_api::resources::params::course_params::UpdateCourseParams;
use canvas_lms_api::resources::params::quiz_params::CreateQuizParams;
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
