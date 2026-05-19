use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn jwt_json() -> serde_json::Value {
    serde_json::json!({
        "token": "eyJhbGciOiJIUzI1NiJ9.test.sig",
        "expires_at": "2026-06-01T01:00:00Z"
    })
}

#[tokio::test]
async fn test_create_jwt() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/jwts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(jwt_json()))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let jwt = canvas.create_jwt().await.unwrap();
    assert_eq!(jwt.token.as_deref(), Some("eyJhbGciOiJIUzI1NiJ9.test.sig"));
    assert!(jwt.expires_at.is_some());
}

#[tokio::test]
async fn test_refresh_jwt() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/jwts/refresh"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "token": "eyJhbGciOiJIUzI1NiJ9.refreshed.sig",
            "expires_at": "2026-06-01T02:00:00Z"
        })))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "token").unwrap();
    let jwt = canvas
        .refresh_jwt("eyJhbGciOiJIUzI1NiJ9.old.sig")
        .await
        .unwrap();
    assert_eq!(
        jwt.token.as_deref(),
        Some("eyJhbGciOiJIUzI1NiJ9.refreshed.sig")
    );
}
