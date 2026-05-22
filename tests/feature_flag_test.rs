use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn flag_json(ctx_type: &str, ctx_id: u64, feature: &str, state: &str) -> serde_json::Value {
    serde_json::json!({
        "feature": feature,
        "context_type": ctx_type,
        "context_id": ctx_id,
        "state": state,
        "locked": false
    })
}

#[tokio::test]
async fn test_feature_flag_delete_course() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/features/flags/new_gradebook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(flag_json(
            "Course",
            1,
            "new_gradebook",
            "on",
        )))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/courses/1/features/flags/new_gradebook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(flag_json(
            "Course",
            1,
            "new_gradebook",
            "off",
        )))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let flag = course.get_feature_flag("new_gradebook").await.unwrap();
    assert_eq!(flag.state.as_deref(), Some("on"));

    let deleted = flag.delete().await.unwrap();
    assert_eq!(deleted.state.as_deref(), Some("off"));
}

#[tokio::test]
async fn test_feature_flag_set_course() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/features/flags/new_gradebook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(flag_json(
            "Course",
            1,
            "new_gradebook",
            "off",
        )))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/courses/1/features/flags/new_gradebook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(flag_json(
            "Course",
            1,
            "new_gradebook",
            "on",
        )))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let course = canvas.get_course(1).await.unwrap();
    let flag = course.get_feature_flag("new_gradebook").await.unwrap();
    assert_eq!(flag.state.as_deref(), Some("off"));

    let updated = flag.set_feature_flag("on").await.unwrap();
    assert_eq!(updated.state.as_deref(), Some("on"));
}

#[tokio::test]
async fn test_feature_flag_delete_account() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/features/flags/new_gradebook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(flag_json(
            "Account",
            1,
            "new_gradebook",
            "on",
        )))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/accounts/1/features/flags/new_gradebook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(flag_json(
            "Account",
            1,
            "new_gradebook",
            "off",
        )))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let account = canvas.get_account(1).await.unwrap();
    let flag = account.get_feature_flag("new_gradebook").await.unwrap();
    let deleted = flag.delete().await.unwrap();
    assert_eq!(deleted.state.as_deref(), Some("off"));
}

#[tokio::test]
async fn test_feature_flag_set_account() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/features/flags/new_gradebook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(flag_json(
            "Account",
            1,
            "new_gradebook",
            "off",
        )))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/accounts/1/features/flags/new_gradebook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(flag_json(
            "Account",
            1,
            "new_gradebook",
            "allowed",
        )))
        .mount(&server)
        .await;

    let canvas = Canvas::new(&server.uri(), "test-token").unwrap();
    let account = canvas.get_account(1).await.unwrap();
    let flag = account.get_feature_flag("new_gradebook").await.unwrap();
    let updated = flag.set_feature_flag("allowed").await.unwrap();
    assert_eq!(updated.state.as_deref(), Some("allowed"));
}
