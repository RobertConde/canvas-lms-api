use canvas_lms_api::{Canvas, CanvasError};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_bad_request() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "errors": [{"message": "invalid"}]
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let err = canvas.get_course(1).await.unwrap_err();

    assert!(
        matches!(err, CanvasError::BadRequest { .. }),
        "expected BadRequest, got {:?}",
        err
    );
}

#[tokio::test]
async fn test_invalid_access_token() {
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
async fn test_unauthorized() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "errors": [{"message": "Unauthorized"}]
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let err = canvas.get_course(1).await.unwrap_err();

    assert!(
        matches!(err, CanvasError::Unauthorized(_)),
        "expected Unauthorized, got {:?}",
        err
    );
}

#[tokio::test]
async fn test_forbidden() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "errors": [{"message": "Forbidden"}]
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let err = canvas.get_course(1).await.unwrap_err();

    assert!(
        matches!(err, CanvasError::Forbidden(_)),
        "expected Forbidden, got {:?}",
        err
    );
}

#[tokio::test]
async fn test_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "errors": [{"message": "The specified resource does not exist."}]
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let err = canvas.get_course(1).await.unwrap_err();

    assert!(
        matches!(err, CanvasError::ResourceDoesNotExist),
        "expected ResourceDoesNotExist, got {:?}",
        err
    );
}

#[tokio::test]
async fn test_conflict() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(409).set_body_json(serde_json::json!({
            "errors": [{"message": "Conflict"}]
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let err = canvas.get_course(1).await.unwrap_err();

    assert!(
        matches!(err, CanvasError::Conflict(_)),
        "expected Conflict, got {:?}",
        err
    );
}

#[tokio::test]
async fn test_unprocessable() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
            "errors": [{"message": "Unprocessable entity"}]
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let err = canvas.get_course(1).await.unwrap_err();

    assert!(
        matches!(err, CanvasError::UnprocessableEntity(_)),
        "expected UnprocessableEntity, got {:?}",
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

    match &err {
        CanvasError::RateLimitExceeded { remaining } => {
            assert_eq!(remaining.as_deref(), Some("0"));
        }
        other => panic!("expected RateLimitExceeded, got {:?}", other),
    }
}
