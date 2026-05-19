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
