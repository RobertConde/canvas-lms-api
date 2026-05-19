use canvas_lms_api::{Canvas, CanvasError};
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

    assert!(matches!(err, CanvasError::ResourceDoesNotExist));
}

#[tokio::test]
async fn test_get_courses_pagination() {
    let server = MockServer::start().await;

    let page2_url = format!("{}/api/v1/courses?page=2&per_page=100", server.uri());

    // First page — Link header points to page 2
    Mock::given(method("GET"))
        .and(path("/api/v1/courses"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header(
                    "Link",
                    format!("<{}>; rel=\"next\"", page2_url),
                )
                .set_body_json(serde_json::json!([
                    {"id": 1, "name": "Course A"},
                    {"id": 2, "name": "Course B"}
                ])),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Second page — no Link header (last page)
    Mock::given(method("GET"))
        .and(wiremock::matchers::query_param("page", "2"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"id": 3, "name": "Course C"}
            ])),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let courses = canvas.get_courses().collect_all().await.unwrap();

    assert_eq!(courses.len(), 3);
    assert_eq!(courses[0].id, 1);
    assert_eq!(courses[1].id, 2);
    assert_eq!(courses[2].id, 3);
}

#[tokio::test]
async fn test_unauthorized() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(
            ResponseTemplate::new(401)
                .insert_header("WWW-Authenticate", "Bearer realm=\"canvas-lms\"")
                .set_body_json(serde_json::json!({
                    "errors": [{"message": "Invalid access token."}]
                })),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "bad-token").unwrap();
    let err = canvas.get_course(1).await.unwrap_err();

    assert!(
        matches!(err, CanvasError::InvalidAccessToken(_)),
        "expected InvalidAccessToken, got {:?}",
        err
    );
}

#[tokio::test]
async fn test_rate_limited() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("X-Rate-Limit-Remaining", "0")
                .set_body_json(serde_json::json!({
                    "errors": [{"message": "Rate limit exceeded."}]
                })),
        )
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let err = canvas.get_course(1).await.unwrap_err();

    assert!(
        matches!(err, CanvasError::RateLimitExceeded { .. }),
        "expected RateLimitExceeded, got {:?}",
        err
    );
}
