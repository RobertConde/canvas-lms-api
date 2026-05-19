use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_get_course() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 1,
                "name": "Introduction to Rust",
                "course_code": "RUST-101",
                "workflow_state": "available",
                "account_id": 10
            })),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();

    assert_eq!(course.id, 1);
    assert_eq!(course.name.as_deref(), Some("Introduction to Rust"));
    assert_eq!(course.course_code.as_deref(), Some("RUST-101"));
    assert_eq!(course.account_id, Some(10));
}

#[tokio::test]
async fn test_get_course_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/9999"))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "errors": [{"message": "The specified resource does not exist."}]
            })),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let err = canvas.get_course(9999).await.unwrap_err();

    assert!(matches!(err, canvas_lms_api::CanvasError::ResourceDoesNotExist));
}
