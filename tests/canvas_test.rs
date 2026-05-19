use canvas_lms_api::resources::params::course_params::CreateCourseParams;
use canvas_lms_api::resources::params::user_params::CreateUserParams;
use canvas_lms_api::resources::user::UserId;
use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_create_course() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/courses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99,
            "name": "New Course",
            "course_code": "NEW-101"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let params = CreateCourseParams {
        name: Some("New Course".to_string()),
        course_code: Some("NEW-101".to_string()),
        ..Default::default()
    };
    let course = canvas.create_course(1, params).await.unwrap();

    assert_eq!(course.id, 99);
    assert_eq!(course.name.as_deref(), Some("New Course"));
    assert_eq!(course.course_code.as_deref(), Some("NEW-101"));
}

#[tokio::test]
async fn test_delete_course() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "Deleted Course",
            "workflow_state": "deleted"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.delete_course(1).await.unwrap();

    assert_eq!(course.id, 1);
    assert!(matches!(
        course.workflow_state,
        Some(canvas_lms_api::resources::types::WorkflowState::Deleted)
    ));
}

#[tokio::test]
async fn test_get_user() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "name": "Alice"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let user = canvas.get_user(UserId::Id(42)).await.unwrap();

    assert_eq!(user.id, 42);
    assert_eq!(user.name.as_deref(), Some("Alice"));
}

#[tokio::test]
async fn test_get_current_user() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/self"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "name": "Bob",
            "effective_locale": "en"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let user = canvas.get_current_user().await.unwrap();

    assert_eq!(user.id, 7);
    assert_eq!(user.name.as_deref(), Some("Bob"));
    assert_eq!(user.effective_locale.as_deref(), Some("en"));
}

#[tokio::test]
async fn test_create_user() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/accounts/1/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 55,
            "name": "Charlie"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let params = CreateUserParams {
        name: "Charlie".to_string(),
        ..Default::default()
    };
    let user = canvas.create_user(1, params).await.unwrap();

    assert_eq!(user.id, 55);
    assert_eq!(user.name.as_deref(), Some("Charlie"));
}

#[tokio::test]
async fn test_get_account() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "Root Account"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let account = canvas.get_account(1).await.unwrap();

    assert_eq!(account.id, 1);
    assert_eq!(account.name.as_deref(), Some("Root Account"));
}

#[tokio::test]
async fn test_get_accounts() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "Root Account"},
            {"id": 2, "name": "Sub Account"}
        ])))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let accounts = canvas.get_accounts().collect_all().await.unwrap();

    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[0].id, 1);
    assert_eq!(accounts[1].id, 2);
}

#[tokio::test]
async fn test_get_section() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/sections/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10, "name": "Section A", "course_id": 1
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let s = canvas.get_section(10).await.unwrap();
    assert_eq!(s.id, 10);
    assert_eq!(s.name.as_deref(), Some("Section A"));
}

#[tokio::test]
async fn test_get_group() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/groups/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20, "name": "Study Group"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let g = canvas.get_group(20).await.unwrap();
    assert_eq!(g.id, 20);
    assert_eq!(g.name.as_deref(), Some("Study Group"));
}

#[tokio::test]
async fn test_get_file() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/files/30"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 30, "display_name": "notes.pdf", "size": 1024
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let f = canvas.get_file(30).await.unwrap();
    assert_eq!(f.id, 30);
    assert_eq!(f.display_name.as_deref(), Some("notes.pdf"));
}

#[tokio::test]
async fn test_get_folder() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/folders/40"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 40, "name": "Homework", "full_name": "course files/Homework"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let f = canvas.get_folder(40).await.unwrap();
    assert_eq!(f.id, 40);
    assert_eq!(f.name.as_deref(), Some("Homework"));
}

#[tokio::test]
async fn test_get_progress() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/progress/50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 50, "workflow_state": "running", "completion": 42
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let p = canvas.get_progress(50).await.unwrap();
    assert_eq!(p.id, 50);
    assert_eq!(p.workflow_state.as_deref(), Some("running"));
}

#[tokio::test]
async fn test_get_outcome() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/outcomes/15"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 15,
            "title": "Written Communication",
            "points_possible": 5.0
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let outcome = canvas.get_outcome(15).await.unwrap();
    assert_eq!(outcome.id, 15);
    assert_eq!(outcome.title.as_deref(), Some("Written Communication"));
    assert_eq!(outcome.points_possible, Some(5.0));
}
