use canvas_lms_api::Canvas;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn feature_json(name: &str) -> serde_json::Value {
    serde_json::json!({
        "feature": name,
        "display_name": "Some Feature",
        "applies_to": "Course",
        "beta": false,
        "development": false,
        "feature_flag": {
            "feature": name,
            "context_type": "Course",
            "context_id": 1,
            "state": "allowed",
            "locked": false
        }
    })
}

fn flag_json(name: &str, state: &str) -> serde_json::Value {
    serde_json::json!({
        "feature": name,
        "context_type": "Course",
        "context_id": 1,
        "state": state,
        "locked": false
    })
}

async fn make_account(server: &MockServer) -> canvas_lms_api::resources::account::Account {
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Canvas::new(&server.uri(), "token")
        .unwrap()
        .get_account(1)
        .await
        .unwrap()
}

async fn make_course(server: &MockServer) -> canvas_lms_api::resources::course::Course {
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
        .mount(server)
        .await;
    Canvas::new(&server.uri(), "token")
        .unwrap()
        .get_course(1)
        .await
        .unwrap()
}

// ── Account features ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_account_get_features() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/features"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            feature_json("course_paces"),
            feature_json("new_gradebook")
        ])))
        .mount(&server)
        .await;

    let account = make_account(&server).await;
    let features = account.get_features().collect_all().await.unwrap();
    assert_eq!(features.len(), 2);
    assert_eq!(features[0].feature.as_deref(), Some("course_paces"));
}

#[tokio::test]
async fn test_account_get_feature_flag() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/features/flags/course_paces"))
        .respond_with(ResponseTemplate::new(200).set_body_json(flag_json("course_paces", "on")))
        .mount(&server)
        .await;

    let account = make_account(&server).await;
    let flag = account.get_feature_flag("course_paces").await.unwrap();
    assert_eq!(flag.feature.as_deref(), Some("course_paces"));
    assert_eq!(flag.state.as_deref(), Some("on"));
}

#[tokio::test]
async fn test_account_get_enabled_features() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/accounts/1/features/enabled"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!(["course_paces", "new_gradebook"])),
        )
        .mount(&server)
        .await;

    let account = make_account(&server).await;
    let enabled = account.get_enabled_features().await.unwrap();
    assert_eq!(enabled.len(), 2);
    assert!(enabled.contains(&"course_paces".to_string()));
}

// ── Course features ───────────────────────────────────────────────────────────

#[tokio::test]
async fn test_course_get_features() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/features"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([feature_json("course_paces")])),
        )
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let features = course.get_features().collect_all().await.unwrap();
    assert_eq!(features.len(), 1);
    assert_eq!(features[0].feature.as_deref(), Some("course_paces"));
}

#[tokio::test]
async fn test_course_get_feature_flag() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/features/flags/course_paces"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(flag_json("course_paces", "allowed")),
        )
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let flag = course.get_feature_flag("course_paces").await.unwrap();
    assert_eq!(flag.state.as_deref(), Some("allowed"));
}

#[tokio::test]
async fn test_course_get_enabled_features() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/courses/1/features/enabled"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(["course_paces"])))
        .mount(&server)
        .await;

    let course = make_course(&server).await;
    let enabled = course.get_enabled_features().await.unwrap();
    assert_eq!(enabled, vec!["course_paces"]);
}
